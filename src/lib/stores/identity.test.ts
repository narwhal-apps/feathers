import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { identity } from './identity.svelte';

const mockInvoke = vi.mocked(invoke);

describe('identity store', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    identity.email = null;
    identity.name = null;
    identity.hydrated = false;
  });

  it('refresh() pulls name + email from settings_get_git_identity', async () => {
    mockInvoke.mockResolvedValueOnce({ name: 'Mike', email: 'mike@example.com' });
    await identity.refresh();
    expect(mockInvoke).toHaveBeenCalledWith('settings_get_git_identity');
    expect(identity.name).toBe('Mike');
    expect(identity.email).toBe('mike@example.com');
    expect(identity.hydrated).toBe(true);
  });

  it('clears name + email if the invoke rejects', async () => {
    identity.name = 'stale';
    identity.email = 'stale@example.com';
    mockInvoke.mockRejectedValueOnce(new Error('no git config'));
    await identity.refresh();
    expect(identity.name).toBeNull();
    expect(identity.email).toBeNull();
    expect(identity.hydrated).toBe(true);
  });

  it('hydrated stays true even after a failure (so consumers stop waiting)', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('boom'));
    await identity.refresh();
    expect(identity.hydrated).toBe(true);
  });
});
