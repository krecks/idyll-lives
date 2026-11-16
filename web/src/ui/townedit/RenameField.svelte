<script lang="ts">
  /**
   * A name typed in a card (a street's, a village's): taken when the field is left or Enter is
   * pressed (`onname`: an edit, refused with why: then the name goes back), Escape puts it back.
   */
  const { value, label, onname }: { value: string; label: string; onname: (name: string) => boolean } = $props();

  // (Another thing picked, or its name changed by an undo: the field follows.)
  let typed = $derived(value);

  function take() {
    if (typed.trim() === value) return;
    if (!onname(typed)) typed = value;
  }
</script>

<input
  class="rename"
  aria-label={label}
  maxlength="40"
  bind:value={typed}
  onblur={take}
  onkeydown={(e) => {
    if (e.key === 'Enter') (e.currentTarget as HTMLInputElement).blur();
    else if (e.key === 'Escape') {
      typed = value;
      e.stopPropagation();
      (e.currentTarget as HTMLInputElement).blur();
    }
  }}
/>

<style>
  .rename {
    width: 100%;
    height: 34px;
    padding: 0 10px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--surface) 70%, transparent);
    color: var(--text-strong);
    font-size: 14px;
    font-weight: 650;
  }
  .rename:focus {
    outline: none;
    background: var(--surface);
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
</style>
