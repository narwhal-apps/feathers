<script lang="ts">
  import SettingsRow from './SettingsRow.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import type { ThemeName } from '$lib/types';

  type Choice = 'system' | ThemeName;

  const current = $derived<Choice>(
    settings.current.theme_override ?? 'system',
  );

  async function pick(value: Choice): Promise<void> {
    const next: ThemeName | null = value === 'system' ? null : value;
    await settings.setTheme(next);
  }

  const options: { id: Choice; label: string }[] = [
    { id: 'system', label: 'System' },
    { id: 'light', label: 'Light' },
    { id: 'dark', label: 'Dark' },
  ];
</script>

<SettingsRow
  label="Appearance"
  description="Match the system, or pin Feathers to a single theme."
>
  {#snippet control()}
    <div class="seg" role="radiogroup" aria-label="Theme">
      {#each options as opt}
        <button
          class="seg-btn"
          class:active={current === opt.id}
          role="radio"
          aria-checked={current === opt.id}
          onclick={() => pick(opt.id)}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  {/snippet}
</SettingsRow>

<style>
  .seg {
    display: inline-flex;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 2px;
  }
  .seg-btn {
    padding: 4px 12px;
    background: transparent;
    border: none;
    border-radius: calc(var(--r-md) - 2px);
    color: var(--fg-muted);
    font-size: var(--fs-xs);
    font-weight: var(--weight-medium);
    cursor: pointer;
    transition: background var(--t-fast), color var(--t-fast);
  }
  .seg-btn:hover { color: var(--fg); }
  .seg-btn.active {
    background: var(--accent-bg-medium);
    color: var(--accent-fg);
    font-weight: var(--weight-semibold);
  }
</style>
