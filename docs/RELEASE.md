# Releasing Feathers

Operator-facing checklist for cutting a release and shipping it through the
in-app updater.

## One-time setup

You only need to do this once per repo (or any time the signing key is
rotated). Estimated time: ~10 minutes.

### 1. Generate the updater signing keypair

```bash
pnpm tauri signer generate -w ~/.tauri/feathers.key
```

This creates two files:

- `~/.tauri/feathers.key` — the **private key** (encrypted with the
  password you choose at the prompt). Used to sign every release.
- `~/.tauri/feathers.key.pub` — the **public key**. Bundled into every
  app build via `tauri.conf.json`. Used by the running app to verify
  that an update bundle came from us.

> **Lose the private key and you cannot publish updates to existing
> installs.** Users would have to manually download + reinstall a new
> build with a new key. Save both the file and the password to your
> password manager **immediately**.

### 2. Commit the public key

Open `src-tauri/tauri.conf.json`. Find:

```json
"plugins": {
  "updater": {
    "pubkey": "REPLACE_WITH_PUBLIC_KEY_FROM_TAURI_SIGNER_GENERATE",
```

Replace the placeholder with the contents of `~/.tauri/feathers.key.pub`
(the entire string, including the headers if `tauri signer` produced
them — it's a single line on Tauri 2). Commit + push.

### 3. Set the GitHub owner in the updater endpoint

Same file, just below the pubkey:

```json
"endpoints": [
  "https://github.com/REPLACE_WITH_GH_OWNER/feathers/releases/latest/download/latest.json"
]
```

Replace `REPLACE_WITH_GH_OWNER` with the actual GitHub user/org that
owns this repo. Commit + push.

### 4. Add GitHub Actions secrets

In the repo's **Settings → Secrets and variables → Actions**, add:

| Name | Value |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | The full contents of `~/.tauri/feathers.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The password you chose during `signer generate` |

Both are read by `.github/workflows/release.yml`'s `tauri-action` step
to sign the update bundles.

---

## Cutting a release

### 1. Bump the version

Three files have to agree on the version string for the updater to
detect "newer":

- `package.json` → `"version": "X.Y.Z"`
- `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"`
- `src-tauri/Cargo.toml` → `version = "X.Y.Z"`

Pre-release tags (`X.Y.Z-rc.1`) work too — the updater compares with
SemVer.

### 2. Update the CHANGELOG

Add a section at the top of `CHANGELOG.md` (create if missing):

```markdown
## vX.Y.Z — YYYY-MM-DD

- Feature / fix bullets
```

### 3. Commit, tag, push

```bash
git commit -am "release: vX.Y.Z"
git push
git tag vX.Y.Z
git push --tags
```

The tag push triggers `.github/workflows/release.yml`. Both
`aarch64-apple-darwin` (Apple Silicon) and `x86_64-apple-darwin`
(Intel) build in parallel and attach their `.dmg`, `.app.tar.gz`, and
`.app.tar.gz.sig` to a single draft release. `latest.json` is also
generated and uploaded.

### 4. Publish the draft

Open https://github.com/<owner>/feathers/releases — there's a draft
release `Feathers vX.Y.Z`. Edit the notes, then click **Publish
release**.

The moment you publish, GitHub flips the `releases/latest` redirect to
this release. Within ~6 hours, every running Feathers app will see the
new version on its background check; users get a toast offering to
install.

---

## What the updater needs (and why)

The updater plugin (`tauri-plugin-updater`) hits the URL configured in
`tauri.conf.json` → `plugins.updater.endpoints`. We point it at:

```
https://github.com/<owner>/feathers/releases/latest/download/latest.json
```

GitHub's `releases/latest` always redirects to the newest *non-draft,
non-prerelease* release. So:

- Drafts are invisible to the updater. Safe to leave a release as a
  draft while you write notes.
- Pre-releases (`prerelease: true` on the GH release) are also
  invisible. Only "stable" releases roll out to existing installs.

Each release needs all of these attached (tauri-action does this
automatically when `includeUpdaterJson: true`):

| File | Purpose |
| --- | --- |
| `Feathers_X.Y.Z_aarch64.dmg` | First-time install for M-series Macs |
| `Feathers_X.Y.Z_x64.dmg` | First-time install for Intel Macs |
| `Feathers.app.tar.gz` (per arch) | The bundle the updater downloads |
| `Feathers.app.tar.gz.sig` (per arch) | Signature the running app verifies against the bundled pubkey |
| `latest.json` | Manifest pointing at the right `.tar.gz` for each platform |

If any of those go missing (e.g. someone manually re-uploaded an
artifact and broke the naming), the updater will silently fail to find
an update.

---

## Adding macOS code signing later

Right now builds are **unsigned**. macOS users see *"this app was
downloaded from the internet, are you sure you want to open it?"* on
first launch. They can right-click → Open to bypass.

To eliminate the warning, you need an Apple Developer account ($99/yr)
and:

1. Generate a Developer ID Application certificate in the Apple
   Developer portal, export as `.p12`
2. Generate an app-specific password in your Apple ID account
3. Add these GitHub Actions secrets:
   - `APPLE_CERTIFICATE` — base64-encoded `.p12`
   - `APPLE_CERTIFICATE_PASSWORD` — `.p12` password
   - `APPLE_SIGNING_IDENTITY` — e.g. `"Developer ID Application: Your Name (TEAMID)"`
   - `APPLE_ID` — your Apple ID email
   - `APPLE_PASSWORD` — the app-specific password from step 2
   - `APPLE_TEAM_ID` — your team ID
4. Uncomment the `APPLE_*` env block in `.github/workflows/release.yml`
5. The next release will be signed + notarised. No more Gatekeeper
   warning.

The updater wiring doesn't change — Apple signing is purely about
*Gatekeeper* trust, while the updater uses our own ed25519 signature.

---

## Troubleshooting

**The release workflow failed with "Resource not accessible by
integration".** The job needs `contents: write` permission. The
workflow already declares it, but if you forked the repo, also check
**Settings → Actions → General → Workflow permissions** is set to
"Read and write".

**The updater says "no update available" even though I just published
v0.2.0.** Check the version string actually changed in all three places
(see Cutting a release step 1). `tauri-plugin-updater` does a strict
SemVer comparison and skips if the published version isn't strictly
greater than the installed one.

**"Signature verification failed" on install.** The bundle was signed
with a different key than the one in the running app's
`tauri.conf.json` `pubkey`. Either you rotated the key (the running
app trusts the old one) or the GH secret is stale. The fix in both
cases is the same: ship a new build that bundles the new pubkey, and
ask users to manually reinstall once.
