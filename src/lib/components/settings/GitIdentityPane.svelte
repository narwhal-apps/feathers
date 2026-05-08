<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import SettingsRow from './SettingsRow.svelte';
  import type { GitIdentity, AppError } from '$lib/types';

  let initial = $state<GitIdentity>({ name: null, email: null });
  let name = $state('');
  let email = $state('');
  let saving = $state(false);
  let errorMsg = $state<string | null>(null);
  let savedFlash = $state(false);

  function formatError(err: unknown): string {
    if (typeof err === 'string') return err;
    const ae = err as AppError;
    if ('message' in ae) return ae.message;
    return String(err);
  }

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
    <input
      type="text"
      placeholder="Your name"
      bind:value={name}
      disabled={saving}
    />
  {/snippet}
</SettingsRow>

<SettingsRow
  label="Email"
  description="Used by GitHub to associate commits with your account."
>
  {#snippet control()}
    <input
      type="email"
      placeholder="you@example.com"
      bind:value={email}
      disabled={saving}
    />
  {/snippet}
</SettingsRow>

<div class="actions">
  {#if errorMsg}<div class="err">Couldn't save: {errorMsg}</div>{/if}
  {#if savedFlash}<div class="ok">Saved.</div>{/if}
  <button class="btn primary" disabled={!dirty || !isValid || saving} onclick={save}>
    {saving ? 'Saving…' : 'Save'}
  </button>
</div>

<style>
  .lede {
    margin: 0 0 var(--sp-3);
    color: var(--fg-subtle);
    font-size: var(--fs-xs);
  }
  input[type="text"], input[type="email"] {
    width: 240px;
    padding: 6px 10px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-sm);
    font-family: var(--font-sans);
  }
  input:focus {
    outline: none;
    border-color: var(--accent-600);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--sp-3);
    margin-top: var(--sp-4);
  }
  .err { color: var(--danger-fg, #c00); font-size: var(--fs-xs); margin-right: auto; }
  .ok { color: var(--accent-fg); font-size: var(--fs-xs); margin-right: auto; }
  .btn {
    padding: 6px 12px;
    background: var(--bg-elev-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-size: var(--fs-xs);
    font-weight: var(--weight-semibold);
    cursor: pointer;
  }
  .btn.primary { background: var(--accent-bg-medium); color: var(--accent-fg); border-color: transparent; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
