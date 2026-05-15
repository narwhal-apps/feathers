import { describe, it, expect, beforeEach, vi } from 'vitest';
import { confirm, notify, _readState, _resolveConfirm, _dismissToast } from './dialog.svelte';

describe('confirm()', () => {
  beforeEach(() => {
    // Drain any toasts/confirms from earlier tests.
    for (const c of _readState().confirms) _resolveConfirm(c.id, false);
    for (const t of _readState().toasts) _dismissToast(t.id);
  });

  it('appends a request to the confirm queue and resolves on _resolveConfirm', async () => {
    const promise = confirm({ title: 'Sure?', message: 'Discard work?' });
    const state = _readState();
    expect(state.confirms.length).toBe(1);
    const id = state.confirms[0].id;
    _resolveConfirm(id, true);
    await expect(promise).resolves.toBe(true);
    expect(_readState().confirms.length).toBe(0);
  });

  it('resolves false when answered negatively', async () => {
    const promise = confirm({ title: 'Sure?', message: 'x' });
    const id = _readState().confirms[0].id;
    _resolveConfirm(id, false);
    await expect(promise).resolves.toBe(false);
  });

  it('queues multiple confirms in order', () => {
    confirm({ title: 'first', message: 'a' });
    confirm({ title: 'second', message: 'b' });
    const state = _readState();
    expect(state.confirms.length).toBe(2);
    expect(state.confirms[0].opts.title).toBe('first');
    expect(state.confirms[1].opts.title).toBe('second');
  });
});

describe('notify()', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    for (const t of _readState().toasts) _dismissToast(t.id);
  });

  it('appends a toast with default kind=info and 3000ms duration', () => {
    notify('hello');
    const t = _readState().toasts[0];
    expect(t.message).toBe('hello');
    expect(t.kind).toBe('info');
    expect(t.durationMs).toBe(3000);
    expect(t.action).toBeUndefined();
  });

  it('respects the kind option', () => {
    notify('boom', { kind: 'error' });
    expect(_readState().toasts[0].kind).toBe('error');
  });

  it('auto-dismisses after the duration', () => {
    notify('hi', { durationMs: 100 });
    expect(_readState().toasts.length).toBe(1);
    vi.advanceTimersByTime(100);
    expect(_readState().toasts.length).toBe(0);
  });

  it('durationMs=0 keeps the toast sticky', () => {
    notify('sticky', { durationMs: 0 });
    expect(_readState().toasts.length).toBe(1);
    vi.advanceTimersByTime(60_000);
    expect(_readState().toasts.length).toBe(1);
  });

  it('forces stickiness when an action is given (regardless of durationMs)', () => {
    notify('Update available', {
      durationMs: 50,
      action: { label: 'Install', onclick: () => {} },
    });
    const t = _readState().toasts[0];
    // durationMs gets overridden to 0 — auto-dismiss timer never fires.
    expect(t.durationMs).toBe(0);
    expect(t.action).toBeDefined();
    vi.advanceTimersByTime(60_000);
    expect(_readState().toasts.length).toBe(1);
  });

  it('_dismissToast removes by id', () => {
    notify('a', { durationMs: 0 });
    notify('b', { durationMs: 0 });
    const ids = _readState().toasts.map((t) => t.id);
    _dismissToast(ids[0]);
    const remaining = _readState().toasts;
    expect(remaining.length).toBe(1);
    expect(remaining[0].message).toBe('b');
  });
});
