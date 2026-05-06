<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/primitives/Icon.svelte';

  let version = $state<string | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    invoke<string>('app_version')
      .then((v) => (version = v))
      .catch((e) => (error = String(e)));
  });
</script>

<svelte:head>
  <title>Feathers</title>
</svelte:head>

<section class="welcome">
  <Icon name="Feather" size={48} />
  <h1>Welcome to Feathers</h1>
  <p class="lead">
    A Tauri-powered desktop Git client. Open or clone a repository to begin.
  </p>
  <p class="version">
    {#if error}
      <span class="err">backend error: {error}</span>
    {:else if version}
      v{version}
    {:else}
      loading…
    {/if}
  </p>
</section>

<style>
  .welcome {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-3);
    color: var(--fg);
    text-align: center;
    padding: var(--sp-6);
  }
  .welcome h1 { margin: 0; font-size: 22px; font-weight: 600; }
  .lead { margin: 0; color: var(--fg-muted); line-height: var(--lh-body); max-width: 420px; }
  .version { color: var(--fg-subtle); font-family: var(--font-mono); font-size: var(--fs-xs); margin: var(--sp-2) 0 0; }
  .err { color: var(--removed); }
</style>
