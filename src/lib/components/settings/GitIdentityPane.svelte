<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import SettingsRow from './SettingsRow.svelte';
  import Button from '$lib/components/primitives/Button.svelte';
  import Input from '$lib/components/primitives/Input.svelte';
  import type { GitIdentity } from '$lib/types';
  import { identity } from '$lib/stores/identity.svelte';
  import { formatError } from '$lib/utils/error';

  let initial = $state<GitIdentity>({ name: null, email: null });
  let name = $state('');
  let email = $state('');
  let saving = $state(false);
  let errorMsg = $state<string | null>(null);
  let savedFlash = $state(false);

  $effect(() => {
    invoke<GitIdentity>('settings_get_git_identity')
      .then((id) => {
        initial = id;
        name = id.name ?? '';
        email = id.email ?? '';
      })
      .catch((err) => { errorMsg = formatError(err); });
  });

  const dirty = $derived(name !== (initial.name ?? '') || email !== (initial.email ?? ''));

  const isValid = $derived.by(() => {
    if (name.trim() === '' && email.trim() === '') return true; // clearing
    if (name.trim() === '' || email.trim() === '') return false;
    const at = email.indexOf('@');
    return at > 0 && email.slice(at + 1).includes('.');
  });

  async function save(): Promise<void> {
    if (!dirty || !isValid) return;
    saving = true;
    errorMsg = null;
    try {
      await invoke('settings_set_git_identity', { name: name.trim(), email: email.trim() });
      initial = { name: name.trim() || null, email: email.trim() || null };
      identity.refresh();
      savedFlash = true;
      setTimeout(() => (savedFlash = false), 1500);
    } catch (err) {
      errorMsg = formatError(err);
    } finally {
      saving = false;
    }
  }
</script>

<p class="lede">Used as the author for commits made in Feathers.</p>

<SettingsRow
  label="Name"
  description="Shows up as the commit author."
>
  {#snippet control()}
    <div class="ctl">
      <Input
        bind:value={name}
        placeholder="Your name"
        disabled={saving}
      />
    </div>
  {/snippet}
</SettingsRow>

<SettingsRow
  label="Email"
  description="Used by GitHub to associate commits with your account."
>
  {#snippet control()}
    <div class="ctl">
      <Input
        type="email"
        bind:value={email}
        placeholder="you@example.com"
        disabled={saving}
      />
    </div>
  {/snippet}
</SettingsRow>

<div class="actions">
  {#if errorMsg}<div class="err">Couldn't save: {errorMsg}</div>{/if}
  {#if savedFlash}<div class="ok">Saved.</div>{/if}
  <Button variant="primary" disabled={!dirty || !isValid || saving} loading={saving} label={saving ? 'Saving…' : 'Save'} onclick={save} />
</div>

<style>
  .lede {
    margin: 0 0 var(--sp-3);
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
  }
  /* Constrain the Input width so it doesn't fill the entire settings row. */
  .ctl { width: 240px; }
  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--sp-3);
    margin-top: var(--sp-4);
  }
  .err { color: var(--removed); font-size: var(--fs-xs); margin-right: auto; }
  .ok { color: var(--accent-fg); font-size: var(--fs-xs); margin-right: auto; }
</style>
