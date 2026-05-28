<script lang="ts">
  type Variant = 'sans' | 'mono';

  let {
    value = $bindable(''),
    placeholder,
    disabled = false,
    required = false,
    rows = 4,
    spellcheck = 'false',
    autocomplete = 'off',
    autocapitalize = 'none',
    variant = 'sans',
    invalid = false,
    autofocus = false,
    resize = 'vertical',
    onkeydown,
    oninput,
    ref = $bindable<HTMLTextAreaElement | null>(null),
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    rows?: number;
    spellcheck?: 'true' | 'false';
    autocomplete?: HTMLTextAreaElement['autocomplete'];
    autocapitalize?: 'none' | 'sentences' | 'words' | 'characters' | 'on' | 'off';
    variant?: Variant;
    invalid?: boolean;
    autofocus?: boolean;
    resize?: 'none' | 'vertical' | 'horizontal' | 'both';
    onkeydown?: (e: KeyboardEvent) => void;
    oninput?: (e: Event) => void;
    ref?: HTMLTextAreaElement | null;
  } = $props();

  $effect(() => { if (autofocus) ref?.focus(); });
</script>

<textarea
  bind:this={ref}
  bind:value
  class="input variant-{variant}"
  class:invalid
  style:resize
  {placeholder}
  {disabled}
  {required}
  {rows}
  {spellcheck}
  {autocomplete}
  {autocapitalize}
  {onkeydown}
  {oninput}
></textarea>

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
    line-height: 1.45;
    outline: none;
    transition: border-color var(--t-fast);
    box-sizing: border-box;
    min-height: 60px;
  }
  .input.variant-mono { font-family: var(--font-mono); }
  .input::placeholder { color: var(--fg-subtle); }
  .input:focus { border-color: var(--accent-500); }
  .input.invalid { border-color: var(--removed); }
  .input.invalid:focus { border-color: var(--removed); }
  .input:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
