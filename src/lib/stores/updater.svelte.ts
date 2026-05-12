/**
 * Frontend wrapper around the Tauri updater plugin.
 *
 * Owns:
 *   - the four-stage state machine (idle / checking / available / downloading
 *     / ready / up-to-date / error) the AboutPane reads from
 *   - the background poller that runs once on app start and re-checks every
 *     six hours while the app stays open
 *   - a sticky `notify()` toast with an "Install" action whenever a check
 *     surfaces a new version
 *
 * Dev mode is a no-op for background checks — the dev binary isn't signed
 * by the updater key, so `check()` would fail noisily on every launch.
 */

import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { notify } from '$lib/utils/dialog.svelte';

const SIX_HOURS_MS = 6 * 60 * 60 * 1000;

export type UpdaterState =
  | { status: 'idle' }
  | { status: 'checking' }
  | { status: 'available'; version: string; notes: string | null }
  | { status: 'downloading'; pct: number }
  | { status: 'ready' }
  | { status: 'up-to-date'; checkedAt: number }
  | { status: 'error'; message: string };

class UpdaterStore {
  state = $state<UpdaterState>({ status: 'idle' });

  /** Cached so download() doesn't have to call check() again. */
  private pending: Update | null = null;
  private timer: ReturnType<typeof setInterval> | null = null;
  private started = false;

  async checkNow(): Promise<void> {
    // Don't re-check while a download is in flight.
    if (this.state.status === 'checking' || this.state.status === 'downloading') return;
    this.state = { status: 'checking' };
    try {
      const update = await check();
      if (update) {
        this.pending = update;
        this.state = {
          status: 'available',
          version: update.version,
          notes: update.body ?? null,
        };
        notify(`Update available — Feathers v${update.version}`, {
          kind: 'info',
          action: { label: 'Install', onclick: () => this.download() },
        });
      } else {
        this.pending = null;
        this.state = { status: 'up-to-date', checkedAt: Date.now() };
      }
    } catch (err) {
      this.pending = null;
      this.state = { status: 'error', message: formatErr(err) };
    }
  }

  async download(): Promise<void> {
    const update = this.pending;
    if (!update) return;
    this.state = { status: 'downloading', pct: 0 };
    let downloaded = 0;
    let total = 0;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          const pct = total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
          // Only push state when the integer pct actually moves — otherwise
          // every 16 KiB chunk re-renders the AboutPane.
          if (this.state.status === 'downloading' && this.state.pct !== pct) {
            this.state = { status: 'downloading', pct };
          }
        } else if (event.event === 'Finished') {
          this.state = { status: 'ready' };
        }
      });
      // downloadAndInstall installs synchronously on macOS once the bundle
      // is downloaded. On success, prompt for relaunch.
      this.state = { status: 'ready' };
      notify('Update installed — relaunch to apply', {
        kind: 'success',
        action: { label: 'Relaunch', onclick: () => relaunch() },
      });
    } catch (err) {
      this.state = { status: 'error', message: formatErr(err) };
    }
  }

  /** Call once on app start. No-op in dev. Idempotent. */
  startBackgroundChecks(): void {
    if (this.started) return;
    this.started = true;
    if (import.meta.env.DEV) return;
    // Initial check after a short delay so the UI has a chance to mount.
    setTimeout(() => { void this.checkNow(); }, 5_000);
    this.timer = setInterval(() => { void this.checkNow(); }, SIX_HOURS_MS);
  }

  stopBackgroundChecks(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.started = false;
  }
}

function formatErr(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;
  return JSON.stringify(err);
}

export const updater = new UpdaterStore();
