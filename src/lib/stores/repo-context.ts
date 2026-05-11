import { getContext, setContext } from 'svelte';
import type { Entry } from '$lib/query/client';
import type { StatusSnapshot, BranchInfo, OpState } from '$lib/types';

const STATUS_KEY = Symbol('repo-status');
const BRANCHES_KEY = Symbol('repo-branches');
const OP_STATE_KEY = Symbol('repo-op-state');

/**
 * Provide the per-repo `repo_status`, `branch_list`, and `repo_op_state`
 * queries from the repo route layout. Children can then read the same
 * reactive entries via `useRepo*` instead of each call site allocating its
 * own subscription. With N consumers, this collapses N notify callbacks into
 * 1 — the cache hash is identical anyway, so there's only ever one IPC
 * fetch, but every extra subscriber re-runs derivations on every notify.
 */
export function provideRepoQueries(qs: {
  status: Entry<StatusSnapshot | null>;
  branches: Entry<BranchInfo[] | null>;
  opState: Entry<OpState | null>;
}): void {
  setContext(STATUS_KEY, qs.status);
  setContext(BRANCHES_KEY, qs.branches);
  setContext(OP_STATE_KEY, qs.opState);
}

export function useRepoStatus(): Entry<StatusSnapshot | null> {
  const v = getContext<Entry<StatusSnapshot | null>>(STATUS_KEY);
  if (!v) throw new Error('useRepoStatus called outside a repo route layout');
  return v;
}

export function useRepoBranches(): Entry<BranchInfo[] | null> {
  const v = getContext<Entry<BranchInfo[] | null>>(BRANCHES_KEY);
  if (!v) throw new Error('useRepoBranches called outside a repo route layout');
  return v;
}

export function useRepoOpState(): Entry<OpState | null> {
  const v = getContext<Entry<OpState | null>>(OP_STATE_KEY);
  if (!v) throw new Error('useRepoOpState called outside a repo route layout');
  return v;
}
