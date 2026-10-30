<script lang="ts" generics="T extends string | number">
  let {
    value = $bindable(),
    options,
    label,
    onchange,
  }: {
    value: T;
    /** `disabled`: shown, not to be picked (arrow keys pass over it). */
    options: readonly { value: T; label: string; disabled?: boolean }[];
    label: string;
    /** Picked (by click or arrow keys). */
    onchange?: (value: T) => void;
  } = $props();

  function pick(v: T) {
    value = v;
    onchange?.(v);
  }

  /** Arrow keys step through the options (a radio group: one tab stop, on the chosen one). */
  function onkeydown(e: KeyboardEvent) {
    const step = e.key === 'ArrowRight' || e.key === 'ArrowDown' ? 1 : e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? -1 : 0;
    if (!step) return;
    e.preventDefault();
    const at = options.findIndex((o) => o.value === value);
    let next = at;
    for (let k = 1; k <= options.length; k++) {
      next = (at + step * k + options.length * k) % options.length;
      if (!options[next].disabled) break;
    }
    if (next === at || options[next].disabled) return;
    pick(options[next].value);
    ((e.currentTarget as HTMLElement).children[next] as HTMLElement | undefined)?.focus();
  }
</script>

<div class="segmented" role="radiogroup" aria-label={label} tabindex="-1" {onkeydown}>
  {#each options as option, i (option.value)}
    <button
      role="radio"
      aria-checked={value === option.value}
      class:active={value === option.value}
      disabled={option.disabled}
      tabindex={value === option.value || (i === 0 && !options.some((o) => o.value === value)) ? 0 : -1}
      onclick={() => pick(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--hairline);
  }
  button {
    height: 30px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-weight: 600;
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  button:disabled {
    opacity: 0.4;
    cursor: default;
  }
  button.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow-sm);
  }
</style>
