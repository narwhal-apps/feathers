import { describe, it, expect } from 'vitest';
import { commitAvatarUrl } from './avatar';

describe('commitAvatarUrl', () => {
  it('returns null when email is missing', () => {
    expect(commitAvatarUrl(null)).toBeNull();
    expect(commitAvatarUrl(undefined)).toBeNull();
    expect(commitAvatarUrl('')).toBeNull();
  });

  describe('GitHub no-reply emails', () => {
    it('parses the post-2017 <id>+<user>@users.noreply.github.com form', () => {
      expect(commitAvatarUrl('12345+mikkri@users.noreply.github.com'))
        .toBe('https://github.com/mikkri.png?size=64');
    });

    it('parses the legacy <user>@users.noreply.github.com form', () => {
      expect(commitAvatarUrl('mikkri@users.noreply.github.com'))
        .toBe('https://github.com/mikkri.png?size=64');
    });

    it('preserves dots/dashes/underscores in the username', () => {
      expect(commitAvatarUrl('foo-bar.baz_qux@users.noreply.github.com'))
        .toBe('https://github.com/foo-bar.baz_qux.png?size=64');
    });

    it('case-insensitive on the host part', () => {
      expect(commitAvatarUrl('mikkri@users.noreply.GITHUB.com'))
        .toBe('https://github.com/mikkri.png?size=64');
    });
  });

  describe('self-email matching', () => {
    it('returns the signed-in avatar when commit email matches local git email', () => {
      const url = commitAvatarUrl('mike@example.com', {
        selfEmail: 'mike@example.com',
        selfAvatarUrl: 'https://avatars.githubusercontent.com/u/1?v=4',
      });
      expect(url).toBe('https://avatars.githubusercontent.com/u/1?v=4');
    });

    it('case-insensitive match on the self email', () => {
      const url = commitAvatarUrl('Mike@Example.COM', {
        selfEmail: 'mike@example.com',
        selfAvatarUrl: 'https://avatars/x.png',
      });
      expect(url).toBe('https://avatars/x.png');
    });

    it('skips the match when selfEmail or selfAvatarUrl is missing', () => {
      expect(commitAvatarUrl('mike@example.com', { selfEmail: 'mike@example.com' })).toBeNull();
      expect(commitAvatarUrl('mike@example.com', { selfAvatarUrl: 'x' })).toBeNull();
    });

    it('falls through to the no-reply branch when self-match misses', () => {
      const url = commitAvatarUrl('12345+other@users.noreply.github.com', {
        selfEmail: 'mike@example.com',
        selfAvatarUrl: 'https://avatars/x.png',
      });
      expect(url).toBe('https://github.com/other.png?size=64');
    });
  });

  it('returns null for arbitrary non-noreply emails', () => {
    expect(commitAvatarUrl('alice@example.com')).toBeNull();
    expect(commitAvatarUrl('foo@gmail.com')).toBeNull();
  });
});
