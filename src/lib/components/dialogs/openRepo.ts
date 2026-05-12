import { open } from '@tauri-apps/plugin-dialog';
import { goto } from '$app/navigation';
import { repos } from '$lib/stores/repos.svelte';
import { notify } from '$lib/utils/dialog.svelte';
import type { AppError } from '$lib/types';

/** Open the OS file picker, then call repo_open and navigate to /repo/[id]/changes. */
export async function openRepoFlow(): Promise<void> {
  const selection = await open({
    multiple: false,
    directory: true,
    title: 'Open Repository',
  });
  if (!selection || Array.isArray(selection)) return;

  try {
    const result = await repos.open(selection);
    repos.activeRepoId = result.id;
    await goto(`/repo/${result.id}/changes/`);
  } catch (err) {
    const e = err as AppError;
    const message =
      e.kind === 'git'
        ? `Not a git repository: ${e.message}`
        : `Failed to open: ${JSON.stringify(err)}`;
    notify(message, { kind: 'error', durationMs: 0 });
  }
}
