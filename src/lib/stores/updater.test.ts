import { describe, it, expect, beforeEach, vi } from 'vitest';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { updater } from './updater.svelte';
import { _readState, _dismissToast } from '$lib/utils/dialog.svelte';

const mockCheck = vi.mocked(check);
const mockRelaunch = vi.mocked(relaunch);

function resetUpdater() {
  // Drop into the private fields to give each test a clean slate.
  // Using the public state setter alone won't reset `pending`/`started`.
  (updater as any).pending = null;
  (updater as any).timer = null;
  (updater as any).started = false;
  updater.state = { status: 'idle' };
}

function clearToasts() {
  for (const t of _readState().toasts) _dismissToast(t.id);
}

describe('updater store', () => {
  beforeEach(() => {
    mockCheck.mockReset();
    mockRelaunch.mockReset();
    resetUpdater();
    clearToasts();
  });

  describe('checkNow()', () => {
    it('idle → up-to-date when check() returns null', async () => {
      mockCheck.mockResolvedValueOnce(null);
      await updater.checkNow();
      expect(updater.state.status).toBe('up-to-date');
      if (updater.state.status === 'up-to-date') {
        expect(updater.state.checkedAt).toBeTypeOf('number');
      }
    });

    it('idle → available when check() returns an Update', async () => {
      mockCheck.mockResolvedValueOnce({
        version: '1.2.3',
        body: 'Bug fixes',
        downloadAndInstall: vi.fn(),
      } as any);
      await updater.checkNow();
      expect(updater.state).toEqual({
        status: 'available',
        version: '1.2.3',
        notes: 'Bug fixes',
      });
    });

    it('queues a sticky "Update available" toast with an Install action', async () => {
      mockCheck.mockResolvedValueOnce({
        version: '2.0.0',
        body: null,
        downloadAndInstall: vi.fn(),
      } as any);
      await updater.checkNow();
      const toasts = _readState().toasts;
      expect(toasts.length).toBe(1);
      expect(toasts[0].message).toContain('v2.0.0');
      // notify() forces durationMs=0 when an action is given.
      expect(toasts[0].durationMs).toBe(0);
      expect(toasts[0].action?.label).toBe('Install');
    });

    it('idle → error when check() rejects', async () => {
      mockCheck.mockRejectedValueOnce(new Error('network down'));
      await updater.checkNow();
      expect(updater.state.status).toBe('error');
      if (updater.state.status === 'error') {
        expect(updater.state.message).toBe('network down');
      }
    });

    it('is a no-op while a check is already in flight', async () => {
      // Hold check() pending to simulate an in-flight checking state.
      let resolve!: (v: any) => void;
      mockCheck.mockReturnValueOnce(new Promise((r) => { resolve = r; }) as any);

      const first = updater.checkNow();
      expect(updater.state.status).toBe('checking');

      // Second call should bail without bumping mockCheck twice.
      await updater.checkNow();
      expect(mockCheck).toHaveBeenCalledTimes(1);

      resolve(null);
      await first;
    });

    it('clears pending when check() returns null', async () => {
      // Plant a pending update first.
      mockCheck.mockResolvedValueOnce({
        version: '1.0.0',
        body: null,
        downloadAndInstall: vi.fn(),
      } as any);
      await updater.checkNow();
      expect((updater as any).pending).not.toBeNull();

      // Dismiss the available-toast before the next check (otherwise the
      // assertion about toast count later would have to account for it).
      clearToasts();

      // Next check → no update.
      mockCheck.mockResolvedValueOnce(null);
      await updater.checkNow();
      expect((updater as any).pending).toBeNull();
    });
  });

  describe('download()', () => {
    it('is a no-op when there is no pending update', async () => {
      await updater.download();
      // State should not have changed away from idle.
      expect(updater.state.status).toBe('idle');
    });

    it('runs through downloading → ready and queues a Relaunch toast', async () => {
      const downloadAndInstall = vi.fn().mockImplementation(async (cb: any) => {
        cb({ event: 'Started', data: { contentLength: 100 } });
        cb({ event: 'Progress', data: { chunkLength: 50 } });
        cb({ event: 'Progress', data: { chunkLength: 50 } });
        cb({ event: 'Finished' });
      });
      mockCheck.mockResolvedValueOnce({
        version: '1.0.0',
        body: null,
        downloadAndInstall,
      } as any);

      await updater.checkNow();
      clearToasts();
      await updater.download();

      expect(downloadAndInstall).toHaveBeenCalledOnce();
      expect(updater.state.status).toBe('ready');

      const toasts = _readState().toasts;
      const relaunchToast = toasts.find((t) => t.action?.label === 'Relaunch');
      expect(relaunchToast).toBeDefined();
      expect(relaunchToast?.durationMs).toBe(0);
    });

    it('the Relaunch toast action calls the relaunch plugin', async () => {
      const downloadAndInstall = vi.fn().mockImplementation(async (cb: any) => {
        cb({ event: 'Finished' });
      });
      mockCheck.mockResolvedValueOnce({
        version: '1.0.0',
        body: null,
        downloadAndInstall,
      } as any);
      await updater.checkNow();
      clearToasts();
      await updater.download();

      const toast = _readState().toasts.find((t) => t.action?.label === 'Relaunch');
      toast?.action?.onclick();
      expect(mockRelaunch).toHaveBeenCalledOnce();
    });

    it('downloading → error when downloadAndInstall rejects', async () => {
      const downloadAndInstall = vi.fn().mockRejectedValue(new Error('disk full'));
      mockCheck.mockResolvedValueOnce({
        version: '1.0.0',
        body: null,
        downloadAndInstall,
      } as any);
      await updater.checkNow();
      await updater.download();

      expect(updater.state.status).toBe('error');
      if (updater.state.status === 'error') {
        expect(updater.state.message).toBe('disk full');
      }
    });
  });
});
