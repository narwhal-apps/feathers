<script lang="ts">
  type InputType = 'text' | 'email' | 'password' | 'url' | 'search' | 'number';
  type Variant = 'sans' | 'mono';

  let {
    value = $bindable(''),
    type = 'text',
    placeholder,
    disabled = false,
    required = false,
    autocomplete = 'off',
    spellcheck = 'false',
    variant = 'sans',
    invalid = false,
    autofocus = false,
    onkeydown,
    oninput,
    ref = $bindable<HTMLInputElement | null>(null),
  }: {
    value?: string;
    type?: InputType;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    autocomplete?: HTMLInputElement['autocomplete'];
    spellcheck?: 'true' | 'false';
    /** mono = code/identifier-style fields (URLs, branch names, paths, SHAs). */
    variant?: Variant;
    invalid?: boolean;
    autofocus?: boolean;
    onkeydown?: (e: KeyboardEvent) => void;
    oninput?: (e: Event) => void;
    ref?: HTMLInputElement | null;
  } = $props();

  $effect(() => { if (autofocus) ref?.focus(); });
</script>

<input
  bind:this={ref}
  bind:value
  class="input variant-{variant}"
  class:invalid
  {type}
  {placeholder}
  {disabled}
  {required}
  {autocomplete}
  {spellcheck}
  {onkeydown}
  {oninput}
/>

<style>
  .input {
    width: 100%;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    color: var(--fg);
    font-family: var(--font-sans);
    font-size: var(--fs-sm);
    line-height: 1.4;
    outline: none;
    transition: border-color var(--t-fast);
    box-sizing: border-box;
  }
  .input.variant-mono { font-family: var(--font-mono); }
  .input::placeholder { color: var(--fg-subtle); }
  .input:focus { border-color: var(--accent-500); }
  .input.invalid { border-color: var(--removed); }
  .input.invalid:focus { border-color: var(--removed); }
  .input:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
