use crate::error::AppError;
use crate::git_core::{op, stage};
use git2::Repository;

/// Bytes from the head of the file we sniff for NUL to decide "this is
/// probably binary, leave it alone." 8 KiB matches `git`'s own heuristic.
const NUL_SCAN_LIMIT: usize = 8 * 1024;

/// Auto-stage every conflicted file whose working-tree contents no longer
/// contain any conflict markers — the same trick GitHub Desktop uses so
/// the user doesn't have to click "Resolved" once they've cleaned a file
/// in their editor. Returns the paths that were just resolved.
///
/// Best-effort: per-file failures (binary, deleted, IO error) are silently
/// skipped so the caller can proceed with the rest of the status read.
pub fn auto_resolve_clean(repo: &Repository) -> Result<Vec<String>, AppError> {
    let conflicted = op::collect_conflicted_paths(repo)?;
    if conflicted.is_empty() {
        return Ok(vec![]);
    }
    let Some(workdir) = repo.workdir() else {
        return Ok(vec![]); // bare repo — nothing to read
    };
    let mut resolved: Vec<String> = vec![];
    for path in &conflicted {
        let abs = workdir.join(path);
        let Ok(bytes) = std::fs::read(&abs) else {
            continue; // delete/modify conflicts and friends — leave to manual
        };
        if is_likely_binary(&bytes) {
            continue;
        }
        if has_conflict_markers(&bytes) {
            continue;
        }
        resolved.push(path.clone());
    }
    if !resolved.is_empty() {
        stage::stage_files(repo, &resolved)?;
    }
    Ok(resolved)
}

fn is_likely_binary(bytes: &[u8]) -> bool {
    let scan_len = bytes.len().min(NUL_SCAN_LIMIT);
    bytes[..scan_len].contains(&0)
}

/// Checks whether the file still carries any of the standard 3-way merge
/// markers (or the diff3-style ancestor marker). Conservative — a single
/// stray marker line is enough to keep the file in the conflicted set,
/// which means the worst case is that the user falls back to clicking
/// "Resolved" manually. Better that than a false-positive resolution.
fn has_conflict_markers(bytes: &[u8]) -> bool {
    // String::from_utf8_lossy avoids allocating when the input is already
    // valid UTF-8 (Cow::Borrowed) — true for essentially all source code.
    for line in String::from_utf8_lossy(bytes).lines() {
        if line.starts_with("<<<<<<<")
            || line.starts_with(">>>>>>>")
            || line.starts_with("|||||||")
            || line.trim_end() == "======="
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_detection_flags_files_with_nul() {
        assert!(is_likely_binary(b"hello\0world"));
        assert!(!is_likely_binary(b"hello world"));
        assert!(!is_likely_binary(b""));
    }

    #[test]
    fn marker_detection_catches_all_three_marker_kinds() {
        let mine = b"line one\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n";
        assert!(has_conflict_markers(mine));

        let only_open = b"<<<<<<< HEAD\nstuff\n";
        assert!(has_conflict_markers(only_open));

        let only_sep = b"alpha\n=======\nbeta\n";
        assert!(has_conflict_markers(only_sep));

        let only_close = b"alpha\n>>>>>>> branch\n";
        assert!(has_conflict_markers(only_close));

        let diff3_base = b"<<<<<<< HEAD\nours\n||||||| ancestor\nbase\n=======\ntheirs\n>>>>>>> b\n";
        assert!(has_conflict_markers(diff3_base));
    }

    #[test]
    fn marker_detection_ignores_clean_files() {
        let clean = b"fn main() {\n    println!(\"hello\");\n}\n";
        assert!(!has_conflict_markers(clean));
    }

    #[test]
    fn marker_detection_ignores_marker_text_mid_line() {
        // Markers must be at the start of a line — `<<<<<<<` inside a
        // string literal or comment doesn't count.
        let weird = b"let s = \"xx <<<<<<< yy\";\n// >>>>>>> end\n";
        // The '// >>>>>>>' line *does* start with `// `, not `>>>>>>>`,
        // so it's correctly ignored.
        assert!(!has_conflict_markers(weird));
    }
}
