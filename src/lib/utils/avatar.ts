/**
 * Best-effort avatar URL for a commit author.
 *
 * Resolution order:
 *   1. If the commit's email matches the local git user.email and we know
 *      the signed-in GitHub user's avatar, use it. Covers our own
 *      commits with a real (non-no-reply) email.
 *   2. GitHub no-reply emails → `https://github.com/<user>.png` (covers
 *      both `<id>+<user>@users.noreply.github.com` and the legacy
 *      `<user>@users.noreply.github.com` form).
 *   3. Otherwise null — the Avatar component falls back to a gradient +
 *      initials, and its onerror handler does the same if the URL 404s.
 */
export function commitAvatarUrl(
  email: string | null | undefined,
  opts: { selfEmail?: string | null; selfAvatarUrl?: string | null } = {},
): string | null {
  if (!email) return null;
  const lower = email.toLowerCase();
  if (
    opts.selfEmail &&
    opts.selfAvatarUrl &&
    lower === opts.selfEmail.toLowerCase()
  ) {
    return opts.selfAvatarUrl;
  }
  const m = email.match(/^(?:\d+\+)?([\w.-]+)@users\.noreply\.github\.com$/i);
  if (m) return `https://github.com/${m[1]}.png?size=64`;
  return null;
}
