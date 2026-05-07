# Feathers

A Tauri 2 + SvelteKit desktop Git client. macOS-only for now.

> Status: Milestone 2 — read-only repo browsing. Open local repos, view status/history/diff. No mutations yet.

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

## Roadmap

See `docs/superpowers/specs/2026-05-06-feathers-mvp-design.md` for the full MVP design and `docs/superpowers/plans/` for milestone plans.
