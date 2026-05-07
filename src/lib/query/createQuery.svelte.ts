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

  $effect(() => {
    const k = key();
    const f = fetcher;
    const sync = () => {
      const e = queryClient.read<T>(k);
      out.data = e?.data;
      out.error = e?.error;
      out.loading = e?.loading ?? false;
    };
    return queryClient.subscribe(k, f, sync);
  });

  return out;
}
