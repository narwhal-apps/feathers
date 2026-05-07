/**
 * Convert a Git remote URL (SSH, HTTPS, or git protocol) to its canonical
 * web form, suitable for browser navigation. Returns null when the URL
 * doesn't look like one we can web-link to (e.g. local file path).
 *
 *   git@github.com:foo/bar.git           → https://github.com/foo/bar
 *   ssh://git@github.com/foo/bar.git     → https://github.com/foo/bar
 *   https://github.com/foo/bar.git       → https://github.com/foo/bar
 *   git://github.com/foo/bar.git         → https://github.com/foo/bar
 */
export function gitUrlToWebUrl(url: string | null | undefined): string | null {
  if (!url) return null;
  const stripGit = (s: string) => s.replace(/\.git\/?$/, '').replace(/\/$/, '');

  // ssh://[user@]host[:port]/path
  const sshProto = url.match(/^ssh:\/\/(?:[^@]+@)?([^:/]+)(?::\d+)?\/(.+)$/);
  if (sshProto) {
    const [, host, path] = sshProto;
    return `https://${host}/${stripGit(path)}`;
  }
  // user@host:path
  const sshShort = url.match(/^[^@\s]+@([^:\s]+):(.+)$/);
  if (sshShort) {
    const [, host, path] = sshShort;
    return `https://${host}/${stripGit(path)}`;
  }
  // git://host/path
  if (url.startsWith('git://')) {
    return `https://${stripGit(url.slice('git://'.length))}`;
  }
  // http(s)://...
  if (/^https?:\/\//.test(url)) {
    return stripGit(url);
  }
  return null;
}

/** `<webBase>/blob/<ref>/<path>`, with each segment URL-encoded. */
export function fileUrlOnRemote(webBase: string, ref: string, path: string): string {
  const encPath = path.split('/').map(encodeURIComponent).join('/');
  return `${webBase}/blob/${encodeURIComponent(ref)}/${encPath}`;
}
