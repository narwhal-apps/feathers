import type { AppError } from '$lib/types';

/** Render any thrown value as user-facing text. Knows the shape of every
 *  `AppError` variant so the user never sees `[object Object]` or raw JSON. */
export function formatError(err: unknown): string {
  if (typeof err === 'string') return err;
  if (err == null) return 'Unknown error';
  if (typeof err !== 'object') return String(err);

  const e = err as AppError;
  switch (e.kind) {
    case 'repo_not_found':
      return `Repository not found: ${e.id}`;
    case 'dirty':
      return `Working tree has uncommitted changes (${e.paths.length} file${e.paths.length === 1 ? '' : 's'}).`;
    case 'merge_conflict':
      return `Merge conflict in ${e.paths.length} file${e.paths.length === 1 ? '' : 's'}.`;
    case 'unmerged':
      return `Branch '${e.name}' has commits not merged into HEAD.`;
    case 'auth':
      return e.message;
    case 'github_rate_limited':
      return `GitHub rate limit hit — try again in ${e.retry_after}s.`;
    case 'not_a_github_repo':
      return 'Origin is not on github.com.';
    case 'network':
      return e.message;
    case 'io':
      return e.message;
    case 'git':
      return e.message;
    case 'cancelled':
      return 'Operation cancelled.';
  }

  // Unknown shape — fall back to JSON, but never to "[object Object]".
  if ('message' in (err as Record<string, unknown>)) {
    return String((err as { message: unknown }).message);
  }
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
