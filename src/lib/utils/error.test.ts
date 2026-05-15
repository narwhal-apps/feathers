import { describe, it, expect } from 'vitest';
import { formatError } from './error';

describe('formatError', () => {
  it('handles strings, null, undefined', () => {
    expect(formatError('boom')).toBe('boom');
    expect(formatError(null)).toBe('Unknown error');
    expect(formatError(undefined)).toBe('Unknown error');
  });

  it('handles primitives via String()', () => {
    expect(formatError(42)).toBe('42');
    expect(formatError(true)).toBe('true');
  });

  it('formats AppError variants', () => {
    expect(formatError({ kind: 'repo_not_found', id: 'abc' }))
      .toBe('Repository not found: abc');
    expect(formatError({ kind: 'dirty', paths: ['a.ts'] }))
      .toBe('Working tree has uncommitted changes (1 file).');
    expect(formatError({ kind: 'dirty', paths: ['a.ts', 'b.ts'] }))
      .toBe('Working tree has uncommitted changes (2 files).');
    expect(formatError({ kind: 'merge_conflict', paths: ['x'] }))
      .toBe('Merge conflict in 1 file.');
    expect(formatError({ kind: 'unmerged', name: 'feat/x' }))
      .toBe("Branch 'feat/x' has commits not merged into HEAD.");
    expect(formatError({ kind: 'auth', message: 'no token' })).toBe('no token');
    expect(formatError({ kind: 'github_rate_limited', retry_after: 60 }))
      .toBe('GitHub rate limit hit — try again in 60s.');
    expect(formatError({ kind: 'not_a_github_repo' }))
      .toBe('Origin is not on github.com.');
    expect(formatError({ kind: 'network', message: 'offline' })).toBe('offline');
    expect(formatError({ kind: 'io', message: 'EPERM' })).toBe('EPERM');
    expect(formatError({ kind: 'git', message: 'reflog' })).toBe('reflog');
    expect(formatError({ kind: 'cancelled' })).toBe('Operation cancelled.');
  });

  it('falls back to .message for unknown shapes', () => {
    expect(formatError({ message: 'something else' })).toBe('something else');
  });

  it('falls back to JSON.stringify when no .message', () => {
    expect(formatError({ random: 'shape' })).toBe('{"random":"shape"}');
  });

  it('never returns "[object Object]"', () => {
    const r = formatError({});
    expect(r).not.toBe('[object Object]');
  });
});
