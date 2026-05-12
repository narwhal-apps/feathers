<script lang="ts">
  import SettingsRow from './SettingsRow.svelte';
  import SegmentedControl from '$lib/components/primitives/SegmentedControl.svelte';
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

  const options: { value: Choice; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
  ];
</script>

<SettingsRow
  label="Appearance"
  description="Match the system, or pin Feathers to a single theme."
>
  {#snippet control()}
    <SegmentedControl
      options={options}
      value={current}
      onChange={pick}
      ariaLabel="Theme"
      size="md"
    />
  {/snippet}
</SettingsRow>
