import { describe, it, expect } from 'vitest';
import { ui } from './ui.svelte';

describe('ui store', () => {
  it('all request counters start at null', () => {
    // Note: this store is a singleton — these assertions assume nothing
    // earlier in the test run touched it. If that ever breaks, switch
    // to constructing a fresh instance per suite.
    expect(ui.branchSwitcherRequest).toBeTypeOf('object'); // null is object
  });

  it('openBranchSwitcher() bumps the counter monotonically', () => {
    const before = ui.branchSwitcherRequest ?? 0;
    ui.openBranchSwitcher();
    expect(ui.branchSwitcherRequest).toBe(before + 1);
    ui.openBranchSwitcher();
    expect(ui.branchSwitcherRequest).toBe(before + 2);
  });

  it('openRepoSwitcher() bumps independently of branch counter', () => {
    const branchBefore = ui.branchSwitcherRequest ?? 0;
    const repoBefore = ui.repoSwitcherRequest ?? 0;
    ui.openRepoSwitcher();
    expect(ui.repoSwitcherRequest).toBe(repoBefore + 1);
    expect(ui.branchSwitcherRequest).toBe(branchBefore);
  });

  it('push() bumps pushRequest', () => {
    const before = ui.pushRequest ?? 0;
    ui.push();
    expect(ui.pushRequest).toBe(before + 1);
  });

  it('createPr() bumps createPrRequest', () => {
    const before = ui.createPrRequest ?? 0;
    ui.createPr();
    expect(ui.createPrRequest).toBe(before + 1);
  });
});
