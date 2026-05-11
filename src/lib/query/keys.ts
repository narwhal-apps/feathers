import type { RepoId } from '$lib/types';

export const queryKeys = {
  knownRepos: () => ['repo', 'known'] as const,
  repoStatus: (id: RepoId) => ['repo', id, 'status'] as const,
  repoBranches: (id: RepoId) => ['repo', id, 'branches'] as const,
  repoLog: (id: RepoId, before?: string | null) =>
    ['repo', id, 'log', before ?? null] as const,
  repoLogUnpushed: (id: RepoId) =>
    ['repo', id, 'log-unpushed'] as const,
  repoDiffWorkdir: (id: RepoId, path: string | null) =>
    ['repo', id, 'diff', 'workdir', path ?? null] as const,
  repoDiffIndex: (id: RepoId, path: string | null) =>
    ['repo', id, 'diff', 'index', path ?? null] as const,
  repoDiffCommit: (id: RepoId, oid: string) =>
    ['repo', id, 'diff', 'commit', oid] as const,
  repoRemoteUrl: (id: RepoId, remote = 'origin') =>
    ['repo', id, 'remote-url', remote] as const,
  repoOpState: (id: RepoId) => ['repo', id, 'op-state'] as const,
  repoPullRequests: (id: RepoId) => ['repo', id, 'pull-requests'] as const,
  repoStashes: (id: RepoId) => ['repo', id, 'stashes'] as const,
  repoStashFiles: (id: RepoId, index: number) =>
    ['repo', id, 'stash', index, 'files'] as const,
  repoStashDiff: (id: RepoId, index: number, path: string) =>
    ['repo', id, 'stash', index, 'diff', path] as const,
};
