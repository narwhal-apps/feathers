/**
 * Cross-component UI signals. Right now: "open the branch switcher" and
 * "open the repo switcher" — fired by global keyboard shortcuts and
 * consumed by the matching dropdown components.
 *
 * The values are monotonically incrementing counters so the consuming
 * `$effect` re-fires every time the shortcut is hit, even if the value
 * was already non-null. `null` means "no request yet this session", so
 * the components don't auto-open on mount.
 */
class UIStore {
  branchSwitcherRequest = $state<number | null>(null);
  repoSwitcherRequest = $state<number | null>(null);
  pushRequest = $state<number | null>(null);
  createPrRequest = $state<number | null>(null);

  openBranchSwitcher(): void {
    this.branchSwitcherRequest = (this.branchSwitcherRequest ?? 0) + 1;
  }
  openRepoSwitcher(): void {
    this.repoSwitcherRequest = (this.repoSwitcherRequest ?? 0) + 1;
  }
  push(): void {
    this.pushRequest = (this.pushRequest ?? 0) + 1;
  }
  createPr(): void {
    this.createPrRequest = (this.createPrRequest ?? 0) + 1;
  }
}

export const ui = new UIStore();
