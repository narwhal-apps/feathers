type Key = readonly (string | number | null)[];
type Fetcher<T> = () => Promise<T>;

export interface Entry<T> {
  data?: T;
  error?: unknown;
  loading: boolean;
  fetcher?: Fetcher<T>;
  subs: Set<() => void>;
}

export class QueryClient {
  private cache = new Map<string, Entry<unknown>>();
  private inflight = new Map<string, Promise<unknown>>();

  private hash(key: Key): string {
    return JSON.stringify(key);
  }

  read<T>(key: Key): Entry<T> | undefined {
    return this.cache.get(this.hash(key)) as Entry<T> | undefined;
  }

  /** Subscribe to a key. Initiates a fetch if no entry exists yet.
   *  `notify` is called whenever data/error/loading changes for this key.
   *  Returns an unsubscribe function. */
  subscribe<T>(key: Key, fetcher: Fetcher<T>, notify: () => void): () => void {
    const k = this.hash(key);
    let entry = this.cache.get(k) as Entry<T> | undefined;
    if (!entry) {
      entry = { loading: false, subs: new Set(), fetcher: fetcher as Fetcher<unknown> as Fetcher<T> };
      this.cache.set(k, entry as Entry<unknown>);
    } else {
      // Keep the latest fetcher so invalidate() uses the most recent closure.
      entry.fetcher = fetcher as Fetcher<unknown> as Fetcher<T>;
    }
    entry.subs.add(notify);
    if (entry.data === undefined && entry.error === undefined && !this.inflight.has(k)) {
      this.fetch(k);
    }
    notify();
    return () => {
      entry!.subs.delete(notify);
      // No GC for now — entries persist. Add LRU later if needed.
    };
  }

  /** Convenience: subscribe + return the fetch promise (for tests). */
  async subscribeAndFetch<T>(key: Key, fetcher: Fetcher<T>, notify: () => void): Promise<void> {
    this.subscribe(key, fetcher, notify);
    const k = this.hash(key);
    const p = this.inflight.get(k);
    if (p) await p;
  }

  /** Invalidate all entries whose key starts with `prefix`.
   *  Active subscriptions trigger a refetch; orphaned entries do nothing.
   *  Stale-while-revalidate: existing `data` is kept visible during the
   *  refetch and only replaced on success. */
  invalidate(prefix: Key): void {
    const prefixHash = JSON.stringify(prefix).slice(0, -1); // drop trailing ']'
    for (const [k, entry] of this.cache.entries()) {
      if (!k.startsWith(prefixHash)) continue;
      entry.error = undefined;
      if (entry.subs.size > 0 && entry.fetcher) {
        this.fetch(k);
      }
    }
  }

  // Convenience: invalidate multiple keys in one call. Usage:
  //   queryClient.invalidateMany([
  //     queryKeys.repoStatus(id),
  //     queryKeys.repoLog(id),
  //   ]);
  invalidateMany(prefixes: Key[]): void {
    for (const p of prefixes) this.invalidate(p);
  }

  private fetch(k: string): void {
    const entry = this.cache.get(k);
    if (!entry || !entry.fetcher) return;
    entry.loading = true;
    entry.subs.forEach((cb) => cb());
    const p = entry.fetcher()
      .then((data) => {
        const e = this.cache.get(k);
        if (!e) return;
        e.data = data;
        e.error = undefined;
        e.loading = false;
        e.subs.forEach((cb) => cb());
      })
      .catch((err) => {
        const e = this.cache.get(k);
        if (!e) return;
        e.error = err;
        e.loading = false;
        e.subs.forEach((cb) => cb());
      })
      .finally(() => {
        this.inflight.delete(k);
      });
    this.inflight.set(k, p);
  }
}

/** Singleton used by createQuery. Replace per-app if multiple FE roots exist. */
export const queryClient = new QueryClient();
