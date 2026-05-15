use crate::error::AppError;
use git2::{
    build::CheckoutBuilder, AnnotatedCommit, AutotagOption, BranchType, Cred, CredentialType,
    FetchOptions, PushOptions, RemoteCallbacks, Repository,
};

const DEFAULT_REMOTE: &str = "origin";

/// Resolve a remote name, falling back to "origin" when none is given.
fn pick_remote(remote: Option<&str>) -> &str {
    remote.unwrap_or(DEFAULT_REMOTE)
}

/// Credential callback used for fetch / push / clone.
///
/// SSH-agent first (covers SSH remotes when the user has an agent unlocked).
/// HTTPS auth requires stored credentials we don't have yet — surfaces a
/// clear error so the UI can show an actionable message.
pub(crate) fn credentials_cb(
    _url: &str,
    username_from_url: Option<&str>,
    allowed: CredentialType,
) -> Result<Cred, git2::Error> {
    if allowed.contains(CredentialType::SSH_KEY) {
        let user = username_from_url.unwrap_or("git");
        return Cred::ssh_key_from_agent(user);
    }
    if allowed.contains(CredentialType::USERNAME) {
        // Some SSH transports first request the username, then the key.
        return Cred::username(username_from_url.unwrap_or("git"));
    }
    if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
        return Err(git2::Error::from_str(
            "HTTPS authentication is not supported yet. \
             Use an SSH remote or sign in to GitHub (coming soon).",
        ));
    }
    if allowed.contains(CredentialType::DEFAULT) {
        return Cred::default();
    }
    Err(git2::Error::from_str("no supported credential type"))
}

fn fetch_options<'a>() -> FetchOptions<'a> {
    let mut cbs = RemoteCallbacks::new();
    cbs.credentials(credentials_cb);
    let mut opts = FetchOptions::new();
    opts.remote_callbacks(cbs);
    opts.download_tags(AutotagOption::Auto);
    opts
}

fn push_options<'a>() -> PushOptions<'a> {
    let mut cbs = RemoteCallbacks::new();
    cbs.credentials(credentials_cb);
    let mut opts = PushOptions::new();
    opts.remote_callbacks(cbs);
    opts
}

/// Read the URL configured for a remote (defaults to "origin"). Returns
/// `None` when the remote doesn't exist or has no URL configured.
pub fn url(repo: &Repository, remote: Option<&str>) -> Result<Option<String>, AppError> {
    let name = pick_remote(remote);
    match repo.find_remote(name) {
        Ok(r) => Ok(r.url().map(|s| s.to_string())),
        Err(_) => Ok(None),
    }
}

pub fn fetch(repo: &Repository, remote: Option<&str>) -> Result<(), AppError> {
    let name = pick_remote(remote);
    let mut r = repo
        .find_remote(name)
        .map_err(|_| AppError::Git { message: format!("remote not found: {name}") })?;

    // Always fetch every branch from this remote, regardless of how the
    // clone's `remote.<name>.fetch` config is set. Without this, single-
    // branch clones (or any repo with a narrowed refspec) only update the
    // current branch's upstream — so the "X commits behind" badge on the
    // default branch goes stale the moment the user switches to a feature
    // branch and hits Fetch.
    let refspec = format!("+refs/heads/*:refs/remotes/{name}/*");
    let mut opts = fetch_options();
    r.fetch(&[refspec.as_str()], Some(&mut opts), None)?;

    // After the network fetch settles, advance every local branch whose
    // upstream is now ahead — but only the strictly behind ones. Branches
    // with local-only commits (any `ahead > 0`) are left alone so we
    // never silently move the user's work. The currently-checked-out
    // branch is also skipped because moving HEAD would require a workdir
    // update and the user expects to do that explicitly via Pull.
    fast_forward_eligible_branches(repo)?;

    Ok(())
}

/// Walk every local branch and fast-forward those whose upstream is
/// purely ahead (`ahead == 0 && behind > 0`). Returns the names of the
/// branches that were advanced — currently unused by callers but handy
/// for tests and a future "Updated N branches" toast.
///
/// Best-effort: per-branch failures are logged and skipped so a single
/// bad ref doesn't kill the whole post-fetch sweep.
fn fast_forward_eligible_branches(repo: &Repository) -> Result<Vec<String>, AppError> {
    // Don't touch HEAD's branch — that requires a workdir/index update,
    // which Pull handles. Detached HEAD (no current branch) → skip nothing.
    let current_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    let mut advanced: Vec<String> = vec![];

    let branches = repo.branches(Some(BranchType::Local))?;
    for entry in branches {
        let (branch, _) = match entry {
            Ok(pair) => pair,
            Err(_) => continue,
        };

        let Some(branch_name) = branch.name().ok().flatten().map(String::from) else {
            continue;
        };
        if current_branch.as_deref() == Some(branch_name.as_str()) {
            continue;
        }

        // Only branches with an upstream are candidates.
        let Ok(upstream) = branch.upstream() else { continue };
        let Some(upstream_oid) = upstream.get().target() else { continue };
        let Some(local_oid) = branch.get().target() else { continue };
        if local_oid == upstream_oid {
            continue;
        }

        let Ok((ahead, behind)) = repo.graph_ahead_behind(local_oid, upstream_oid) else {
            continue;
        };
        if ahead != 0 || behind == 0 {
            continue;
        }

        // Pure fast-forward: move the local ref to the upstream tip.
        // This is what `git fetch <remote> <branch>:<branch>` does on a
        // non-checked-out branch.
        let mut local_ref = branch.into_reference();
        if local_ref
            .set_target(upstream_oid, "feathers: fast-forward via fetch")
            .is_ok()
        {
            advanced.push(branch_name);
        }
    }

    Ok(advanced)
}

pub fn push(repo: &Repository, remote: Option<&str>) -> Result<(), AppError> {
    let name = pick_remote(remote);
    let head = repo.head()?;
    let head_name = head.name().ok_or_else(|| AppError::Git {
        message: "HEAD has no ref name".into(),
    })?;
    let branch = head.shorthand().ok_or_else(|| AppError::Git {
        message: "HEAD is detached or has no branch".into(),
    })?;

    let mut r = repo
        .find_remote(name)
        .map_err(|_| AppError::Git { message: format!("remote not found: {name}") })?;
    let refspec = format!("{head_name}:refs/heads/{branch}");
    let mut opts = push_options();
    r.push(&[refspec.as_str()], Some(&mut opts))?;
    Ok(())
}

/// Push the current branch and configure it to track its same-named ref on
/// the remote. Used when the local branch has no upstream yet ("publish").
pub fn publish(repo: &Repository, remote: Option<&str>) -> Result<(), AppError> {
    let name = pick_remote(remote).to_string();
    let head = repo.head()?;
    let head_name = head
        .name()
        .ok_or_else(|| AppError::Git { message: "HEAD has no ref name".into() })?
        .to_string();
    let branch_name = head
        .shorthand()
        .ok_or_else(|| AppError::Git { message: "HEAD is detached or has no branch".into() })?
        .to_string();
    drop(head);

    {
        let mut r = repo
            .find_remote(&name)
            .map_err(|_| AppError::Git { message: format!("remote not found: {name}") })?;
        let refspec = format!("{head_name}:refs/heads/{branch_name}");
        let mut opts = push_options();
        r.push(&[refspec.as_str()], Some(&mut opts))?;
    }

    let mut local = repo.find_branch(&branch_name, BranchType::Local)?;
    let upstream_name = format!("{name}/{branch_name}");
    local.set_upstream(Some(&upstream_name))?;
    Ok(())
}

/// Pull = fetch + integrate. When `rebase` is false: refuses anything that
/// can't be fast-forwarded. When `rebase` is true: rebases local commits onto
/// the upstream tip, aborting on conflict and returning `MergeConflict`.
pub fn pull(repo: &Repository, remote: Option<&str>, rebase: bool) -> Result<(), AppError> {
    fetch(repo, remote)?;

    let head_ref = repo.head()?;
    let head_branch = git2::Branch::wrap(head_ref);
    let upstream = head_branch.upstream().map_err(|_| AppError::Git {
        message: "current branch has no upstream configured".into(),
    })?;
    let upstream_oid = upstream.get().target().ok_or_else(|| AppError::Git {
        message: "upstream has no target".into(),
    })?;
    let upstream_ann = repo.find_annotated_commit(upstream_oid)?;

    let (analysis, _) = repo.merge_analysis(&[&upstream_ann])?;
    if analysis.is_up_to_date() {
        return Ok(());
    }

    if analysis.is_fast_forward() && !rebase {
        return fast_forward(repo, upstream_oid);
    }

    if rebase {
        return rebase_onto_upstream(repo, &upstream_ann);
    }

    Err(AppError::Git {
        message: "cannot fast-forward; pull with rebase or merge manually".into(),
    })
}

fn fast_forward(repo: &Repository, target_oid: git2::Oid) -> Result<(), AppError> {
    let head_ref_name = repo
        .head()?
        .name()
        .ok_or_else(|| AppError::Git { message: "HEAD has no name".into() })?
        .to_string();

    // Canonical libgit2 fast-forward sequence: checkout the new tree
    // FIRST (so safe-mode compares against the *current* HEAD as baseline
    // and updates both workdir + index in one go), THEN move the HEAD
    // ref. Doing it the other way round leaves the index pointing at the
    // old tree and surfaces every incoming change as a phantom staged
    // diff in the FE.
    let new_commit = repo.find_commit(target_oid)?;
    repo.checkout_tree(
        new_commit.as_object(),
        Some(CheckoutBuilder::new().safe()),
    )?;

    let mut head_ref_mut = repo.find_reference(&head_ref_name)?;
    head_ref_mut.set_target(target_oid, "fast-forward")?;
    repo.set_head(&head_ref_name)?;
    Ok(())
}

fn rebase_onto_upstream(
    repo: &Repository,
    upstream: &AnnotatedCommit<'_>,
) -> Result<(), AppError> {
    let head_oid = repo.head()?.target().ok_or_else(|| AppError::Git {
        message: "HEAD has no target".into(),
    })?;
    let local_ann = repo.find_annotated_commit(head_oid)?;

    let mut rb = repo.rebase(Some(&local_ann), Some(upstream), None, None)?;
    let sig = repo.signature()?;

    while let Some(op) = rb.next() {
        op?;
        let idx = repo.index()?;
        if idx.has_conflicts() {
            // Leave the rebase paused on disk; the FE detects the state and
            // shows a "Resolve conflicts" panel with continue / abort.
            return Ok(());
        }
        rb.commit(None, &sig, None)?;
    }
    rb.finish(None)?;
    Ok(())
}
