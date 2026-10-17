<script lang="ts">
  import { services } from '../services';

  /**
   * An object's colour: the content's palette (`objectColors`), "as made" and any colour of the
   * player's own. Only for things whose model paints some of its parts (`paintable`).
   */
  let { color = null, onpick }: { /** `#rrggbb`, or null: as made. */ color?: string | null; onpick: (color: string | null) => void } = $props();

  const palette = services.content.objectColors;
  const current = $derived(color?.toLowerCase() ?? null);
  const named = $derived(palette.find((c) => c.color.toLowerCase() === current));
  /** The colour input starts from the colour it has (white: as made). */
  const own = $derived(current ?? '#ffffff');

  /**
   * The colour input: the system's colour panel reports every colour while the player drags through
   * it (`input`), and `change` only once it closes (macOS). Each pause while dragging gives the
   * object that colour, so it changes as they choose, without a recolour for every step.
   */
  const SETTLE_MS = 250;
  let pending: ReturnType<typeof setTimeout> | null = null;
  function ownColor(value: string, now: boolean) {
    if (pending) clearTimeout(pending);
    pending = null;
    const color = value.toLowerCase();
    if (color === current) return;
    if (now) onpick(color);
    else pending = setTimeout(() => ((pending = null), onpick(color)), SETTLE_MS);
  }
  $effect(() => () => {
    if (pending) clearTimeout(pending);
  });
</script>

<div class="colors">
  <div class="swatches" role="radiogroup" aria-label="Colour">
    <button role="radio" aria-checked={current === null} class="swatch made" class:active={current === null} title="As made" onclick={() => onpick(null)}>
      <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true"><path d="M3 13 13 3" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" /></svg>
    </button>
    {#each palette as c (c.id)}
      <button role="radio" aria-checked={current === c.color.toLowerCase()} class="swatch" class:active={current === c.color.toLowerCase()} style="--c:{c.color}" title={c.label} onclick={() => onpick(c.color.toLowerCase())}></button>
    {/each}
    <label class="swatch own" class:active={current !== null && !named} title="A colour of your own" style="--c:{current !== null && !named ? current : 'transparent'}">
      <input type="color" value={own} oninput={(e) => ownColor(e.currentTarget.value, false)} onchange={(e) => ownColor(e.currentTarget.value, true)} aria-label="A colour of your own" />
      <span aria-hidden="true">+</span>
    </label>
  </div>
  <span class="chosen"><b>{current === null ? 'As made' : (named?.label ?? current.toUpperCase())}</b><small> · free, it works the same</small></span>
</div>

<style>
  .colors {
    display: grid;
    gap: 4px;
  }
  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 3px 2px;
  }
  .swatch {
    flex: none;
    width: 24px;
    height: 24px;
    border-radius: 7px;
    background: var(--c);
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.12);
    transition: transform 160ms var(--ease);
  }
  .swatch:hover {
    transform: translateY(-2px) scale(1.08);
  }
  .swatch.active {
    box-shadow:
      inset 0 0 0 1px rgba(0, 0, 0, 0.12),
      0 0 0 2px var(--surface),
      0 0 0 4px var(--accent, var(--build));
  }
  .swatch.made,
  .swatch.own {
    display: grid;
    place-items: center;
    color: var(--text-muted);
    font-weight: 750;
    font-size: 13px;
  }
  .swatch.made {
    background: var(--surface);
  }
  .swatch.own {
    position: relative;
    cursor: pointer;
    background:
      linear-gradient(var(--c), var(--c)),
      conic-gradient(#e85d5d, #e8c25d, #8ed36b, #5dc7e8, #7a6be8, #e85dc4, #e85d5d);
  }
  .swatch.own input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .swatch.own span {
    text-shadow: 0 0 3px var(--surface);
  }
  .chosen small {
    color: var(--text-muted);
  }
</style>
