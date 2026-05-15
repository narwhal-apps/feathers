import { describe, it, expect, vi } from 'vitest';
import { QueryClient } from './client';

describe('QueryClient', () => {
  it('dedupes concurrent fetches for the same key', async () => {
    const qc = new QueryClient();
    const fetcher = vi.fn().mockResolvedValue('hello');
    const p1 = qc.subscribeAndFetch(['k'], fetcher, () => {});
    const p2 = qc.subscribeAndFetch(['k'], fetcher, () => {});
    await Promise.all([p1, p2]);
    expect(fetcher).toHaveBeenCalledTimes(1);
  });

  it('invalidates by prefix and refetches active subscriptions', async () => {
    const qc = new QueryClient();
    let n = 0;
    const fetcher = vi.fn().mockImplementation(() => Promise.resolve(++n));
    let observed = 0;
    qc.subscribe(['repo', 'a', 'status'], fetcher, () => {
      observed = qc.read<number>(['repo', 'a', 'status'])!.data!;
    });
    await new Promise((r) => setTimeout(r, 0));
    expect(observed).toBe(1);
    qc.invalidate(['repo', 'a']);
    await new Promise((r) => setTimeout(r, 0));
    expect(observed).toBe(2);
  });

  it('coalesces overlapping invalidations into one trailing refetch', async () => {
    const qc = new QueryClient();
    let started = 0;
    let resolveCurrent: ((v: number) => void) | null = null;
    const fetcher = vi.fn().mockImplementation(() => {
      started++;
      return new Promise<number>((res) => { resolveCurrent = res; });
    });
    qc.subscribe(['k'], fetcher, () => {});
    expect(started).toBe(1);

    // Three invalidations while the first fetch is still in flight should
    // not spawn three more fetcher() calls — they collapse into one
    // trailing refetch that runs once the in-flight one settles.
    qc.invalidate(['k']);
    qc.invalidate(['k']);
    qc.invalidate(['k']);
    expect(started).toBe(1);

    resolveCurrent!(1);
    await new Promise((r) => setTimeout(r, 0));
    expect(started).toBe(2);

    // Once the trailing refetch is in flight, no further coalesced one
    // should fire if no new invalidate has arrived.
    resolveCurrent!(2);
    await new Promise((r) => setTimeout(r, 0));
    expect(started).toBe(2);
  });

  it('does not run trailing refetch when last subscriber unsubscribes mid-flight', async () => {
    const qc = new QueryClient();
    let started = 0;
    let resolveCurrent: ((v: number) => void) | null = null;
    const fetcher = vi.fn().mockImplementation(() => {
      started++;
      return new Promise<number>((res) => { resolveCurrent = res; });
    });
    const unsub = qc.subscribe(['k'], fetcher, () => {});
    expect(started).toBe(1);
    qc.invalidate(['k']);     // queues a trailing refetch
    unsub();                  // last sub leaves before in-flight settles
    resolveCurrent!(1);
    await new Promise((r) => setTimeout(r, 0));
    // Trailing refetch should be suppressed — nobody is listening.
    expect(started).toBe(1);
  });

  it('does not refetch invalidated entries with no active subs', async () => {
    const qc = new QueryClient();
    const fetcher = vi.fn().mockResolvedValue(1);
    const unsub = qc.subscribe(['k'], fetcher, () => {});
    await new Promise((r) => setTimeout(r, 0));
    unsub();
    qc.invalidate(['k']);
    await new Promise((r) => setTimeout(r, 0));
    expect(fetcher).toHaveBeenCalledTimes(1);
  });
});
