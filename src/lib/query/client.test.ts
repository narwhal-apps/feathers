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
