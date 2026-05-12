/**
 * Best-effort avatar URL for a commit author.
 *
 * Recognises GitHub no-reply email formats and constructs the user's
 * github.com profile picture URL. Returns null otherwise — the Avatar
 * component falls back to a gradient + initials, and its onerror
 * handler will do the same if the URL 404s.
 *
 * Formats supported:
 *   <id>+<username>@users.noreply.github.com   (default since 2017)
 *   <username>@users.noreply.github.com        (legacy)
 */
export function commitAvatarUrl(
  email: string | null | undefined,
): string | null {
  if (!email) return null;
  const m = email.match(/^(?:\d+\+)?([\w.-]+)@users\.noreply\.github\.com$/i);
  if (m) return `https://github.com/${m[1]}.png?size=64`;
  return null;
}
