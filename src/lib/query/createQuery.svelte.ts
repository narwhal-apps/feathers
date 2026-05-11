import { queryClient, type Entry } from './client';

type Key = readonly (string | number | null)[];

/** Svelte 5 reactive query. Caller holds the returned object;
 *  reading `.data`, `.loading`, `.error` is reactive.
 *
 *  Both `key` and `fetcher` are reactive — calling site can pass arrow
 *  functions that read $state values. */
export function createQuery<T>(
  key: () => Key,
  fetcher: () => Promise<T>,
) {
  const out = $state<Entry<T>>({ loading: true, subs: new Set() });

  // Memoize the key by its hash so the effect only resubscribes when the
  // key contents actually change — not on every parent re-render that
  // produces a new array with identical contents. Match queryClient's
  // internal hashing (JSON.stringify) for consistency.
  const k = $derived(key());
  const kHash = $derived(JSON.stringify(k));

  $effect(() => {
    // Track only kHash, not the array identity of k. Identical successive
    // keys produce the same hash string, so the effect doesn't re-run.
    void kHash;
    const sync = () => {
      const e = queryClient.read<T>(k);
      out.data = e?.data;
      out.error = e?.error;
      out.loading = e?.loading ?? false;
    };
    return queryClient.subscribe(k, fetcher, sync);
  });

  return out;
}
