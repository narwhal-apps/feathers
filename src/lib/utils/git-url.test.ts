import { describe, it, expect } from 'vitest';
import { gitUrlToWebUrl, fileUrlOnRemote } from './git-url';

describe('gitUrlToWebUrl', () => {
  it('handles SSH short form (user@host:path)', () => {
    expect(gitUrlToWebUrl('git@github.com:foo/bar.git')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('git@github.com:foo/bar')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('git@gitlab.com:org/sub/repo.git')).toBe('https://gitlab.com/org/sub/repo');
  });

  it('handles ssh:// protocol', () => {
    expect(gitUrlToWebUrl('ssh://git@github.com/foo/bar.git')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('ssh://git@github.com:22/foo/bar.git')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('ssh://github.com/foo/bar.git')).toBe('https://github.com/foo/bar');
  });

  it('handles git:// protocol', () => {
    expect(gitUrlToWebUrl('git://github.com/foo/bar.git')).toBe('https://github.com/foo/bar');
  });

  it('handles https:// (the easy case)', () => {
    expect(gitUrlToWebUrl('https://github.com/foo/bar.git')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('https://github.com/foo/bar')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('https://github.com/foo/bar/')).toBe('https://github.com/foo/bar');
    expect(gitUrlToWebUrl('http://example.com/foo')).toBe('http://example.com/foo');
  });

  it('strips trailing slash from .git removal', () => {
    expect(gitUrlToWebUrl('git@github.com:foo/bar.git/')).toBe('https://github.com/foo/bar');
  });

  it('returns null for unrecognised inputs', () => {
    expect(gitUrlToWebUrl(null)).toBeNull();
    expect(gitUrlToWebUrl(undefined)).toBeNull();
    expect(gitUrlToWebUrl('')).toBeNull();
    expect(gitUrlToWebUrl('/local/path/repo')).toBeNull();
    expect(gitUrlToWebUrl('file:///some/path')).toBeNull();
  });
});

describe('fileUrlOnRemote', () => {
  it('builds a /blob/ URL with each segment encoded', () => {
    expect(fileUrlOnRemote('https://github.com/foo/bar', 'main', 'src/index.ts'))
      .toBe('https://github.com/foo/bar/blob/main/src/index.ts');
  });

  it('encodes ref + path segments', () => {
    expect(fileUrlOnRemote('https://github.com/foo/bar', 'feat/x y', 'a/b c.ts'))
      .toBe('https://github.com/foo/bar/blob/feat%2Fx%20y/a/b%20c.ts');
  });

  it('handles SHA refs', () => {
    expect(fileUrlOnRemote('https://github.com/foo/bar', 'abc123', 'README.md'))
      .toBe('https://github.com/foo/bar/blob/abc123/README.md');
  });
});
