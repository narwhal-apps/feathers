# Feathers

A Tauri 2 + SvelteKit desktop Git client. macOS-only for now.

> Status: post-MVP. Open local repos, browse status/history/diff, stage/commit/discard, branch, cherry-pick/revert/reset, stash, push/pull/fetch, GitHub sign-in + PR list/create, settings window.

## Prerequisites

- pnpm 9+
- Rust (stable)
- Xcode command-line tools (`xcode-select --install`)

## Develop

```bash
pnpm install
pnpm tauri dev
```

## Test

```bash
pnpm test                    # frontend unit/component tests (vitest)
cd src-tauri && cargo test   # backend tests
```

## Type-check & lint

```bash
pnpm check                                   # svelte-check
cd src-tauri && cargo clippy -- -D warnings  # rust lints
cd src-tauri && cargo fmt                    # rust format
```

## Repository state files we write

Feathers stores its own per-repo state under `.git/feathers/` so that nothing leaks outside the repo's `.git` dir:

- `STASH_APPLY.json` — sidecar marker for an in-flight `stash apply`/`pop`. Used by the Resolve panel to drive Continue/Abort and to drop the stash on a successful pop. Removed automatically after the apply settles. Safe to delete by hand if it ever gets stranded.
