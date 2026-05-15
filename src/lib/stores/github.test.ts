import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { github } from './github.svelte';

const mockInvoke = vi.mocked(invoke);

describe('github store', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    github.user = null;
    github.loading = false;
    github.hydrated = false;
  });

  it('refresh() loads user and flips hydrated/loading', async () => {
    mockInvoke.mockResolvedValueOnce({
      login: 'mikkri',
      avatar_url: 'https://x/y.png',
    });
    const p = github.refresh();
    expect(github.loading).toBe(true);
    await p;
    expect(mockInvoke).toHaveBeenCalledWith('github_user');
    expect(github.user?.login).toBe('mikkri');
    expect(github.hydrated).toBe(true);
    expect(github.loading).toBe(false);
  });

  it('refresh() resolves user=null when invoke throws (still hydrated)', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('offline'));
    await github.refresh();
    expect(github.user).toBeNull();
    expect(github.hydrated).toBe(true);
    expect(github.loading).toBe(false);
  });

  it('refresh() accepts a null user from the backend (signed-out state)', async () => {
    mockInvoke.mockResolvedValueOnce(null);
    await github.refresh();
    expect(github.user).toBeNull();
    expect(github.hydrated).toBe(true);
  });

  it('signOut() invokes github_signout and clears user', async () => {
    github.user = { login: 'mikkri', avatar_url: 'x' } as any;
    mockInvoke.mockResolvedValueOnce(undefined);
    await github.signOut();
    expect(mockInvoke).toHaveBeenCalledWith('github_signout');
    expect(github.user).toBeNull();
  });
});
