<script lang="ts">
  import { paintSlots } from '../../assets/paint';
  import type { PartColors } from '../../core/protocol';
  import type { ItemStage } from '../../render/types';
  import { services } from '../services';
  import { play } from '../sfx';
  import { game } from '../state.svelte';
  import { withColor } from './catalog';
  import ColorSwatches from './ColorSwatches.svelte';
  import Turntable from './Turntable.svelte';

  /**
   * The colour studio, over Build mode: the item live and turning on the left, its parts on the
   * right, each with the palette, "as made" and a colour of the player's own. Nothing changes in
   * the world until Done: then a thing at home is recoloured (one edit, one undo) or the new
   * purchase in hand takes the colours. Cancel (or Escape) leaves it as it was.
   */
  let { def: defId, objectId }: { def: string; objectId: number | null } = $props();

  const content = services.content;
  const def = $derived(content.object(defId));
  const owned = $derived(objectId === null ? null : (game.objects.find((o) => o.id === objectId) ?? null));
  const slots = $derived(def ? paintSlots(services.assets.get(def.model, 'model')) : []);
  const palette = content.objectColors;

  /** The colours as they were (what Cancel keeps), and as the player has them now. */
  const before: PartColors = (() => {
    const placed = objectId === null ? game.placing?.colors : game.objects.find((o) => o.id === objectId)?.colors;
    return { ...placed };
  })();
  let draft = $state<PartColors>({ ...before });
  let part = $state<string | null>(null);
  const chosen = $derived(part ?? slots[0]?.material ?? null);
  const changed = $derived(JSON.stringify(sortedColors(draft)) !== JSON.stringify(sortedColors(before)));
  const coloured = $derived(slots.filter((s) => draft[s.material]).length);

  function sortedColors(c: PartColors): [string, string][] {
    return Object.entries(c).sort(([a], [b]) => a.localeCompare(b));
  }

  function colorName(color: string | undefined): string {
    if (!color) return 'As made';
    return palette.find((c) => c.color.toLowerCase() === color.toLowerCase())?.label ?? color.toUpperCase();
  }

  function pick(material: string, color: string | null) {
    draft = withColor(draft, material, color) ?? {};
  }

  function close() {
    game.customising = null;
  }

  function done() {
    if (changed) {
      const colors = Object.keys(draft).length ? $state.snapshot(draft) : undefined;
      if (owned) services.controls.recolor(owned.id, colors);
      else services.controls.colorPlacing(colors);
      play('paint');
    }
    close();
  }

  // The thing went (sold, undone) or left the hand: nothing to colour any more.
  $effect(() => {
    if (objectId !== null ? !owned : game.placing?.objectId !== null || game.placing?.def !== defId) close();
  });

  // The live stage: the item in the draft's colours. Without 3D, the turntable pictures instead.
  let host = $state<HTMLElement | null>(null);
  let stage = $state.raw<ItemStage | null>(null);
  let flat = $state(false);
  $effect(() => {
    const el = host;
    const d = def;
    if (!el || !d) return;
    let live = true;
    let made: ItemStage | null = null;
    void services.items.stage(el, d.model, d.footprint ?? [1, 1]).then((s) => {
      if (!live) return s?.detach();
      made = s;
      if (s) stage = s;
      else flat = true;
    });
    return () => {
      live = false;
      made?.detach();
      stage = null;
    };
  });
  $effect(() => {
    stage?.setColors(Object.keys(draft).length ? $state.snapshot(draft) : undefined);
  });
</script>

{#if def}
  <div class="studio" role="dialog" aria-modal="true" aria-label="Colours of the {def.name}">
    <div class="stage" bind:this={host}>
      {#if flat}
        <div class="flat"><Turntable model={def.model} footprint={def.footprint ?? [1, 1]} glyph="icon.ui.buy" colors={draft} /></div>
      {:else}
        <span class="hint">↔ drag to turn · scroll to come closer</span>
      {/if}
      <button class="btn ghost small reset" disabled={!Object.keys(draft).length} title="Every part as it's made" onclick={() => (draft = {})}>Reset all</button>
    </div>

    <aside class="panel glass">
      <header>
        <span class="eyebrow">Colours</span>
        <b class="name">{def.name}</b>
        <span class="muted">{coloured ? `${coloured} of ${slots.length} parts coloured` : 'As made'} · free, it works the same</span>
      </header>

      <div class="parts" role="radiogroup" aria-label="Parts">
        {#each slots as slot (slot.material)}
          {@const color = draft[slot.material]}
          {@const open = chosen === slot.material}
          <div class="part" class:open>
            <button role="radio" aria-checked={open} class="row" onclick={() => (part = slot.material)}>
              <span class="chip" class:made={!color} style="--c:{color ?? 'transparent'}" aria-hidden="true">
                {#if !color}<svg viewBox="0 0 16 16" width="12" height="12"><path d="M3 13 13 3" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" /></svg>{/if}
              </span>
              <span class="label">{slot.label}</span>
              <span class="value muted">{colorName(color)}</span>
            </button>
            {#if open}
              <div class="pick">
                <ColorSwatches color={color ?? null} onpick={(c) => pick(slot.material, c)} />
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <footer>
        <button class="btn ghost" onclick={close}>Cancel</button>
        <button class="btn primary" onclick={done}>Done</button>
      </footer>
    </aside>
  </div>
{/if}

<style>
  /* Over the game (and its interface), which shows dimmed behind the studio. */
  .studio {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    grid-template-columns: minmax(0, 1fr) min(400px, 42vw);
    gap: 16px;
    padding: 16px;
    background: linear-gradient(180deg, rgba(243, 236, 226, 0.86), rgba(214, 206, 194, 0.92));
    backdrop-filter: blur(6px);
  }
  .stage {
    position: relative;
    min-height: 0;
    border-radius: 18px;
    background: radial-gradient(ellipse at 50% 62%, rgba(255, 255, 255, 0.75), rgba(255, 255, 255, 0) 70%);
    overflow: hidden;
  }
  .flat {
    position: absolute;
    inset: 10% 15%;
    display: grid;
    place-items: center;
  }
  .hint {
    position: absolute;
    left: 50%;
    bottom: 14px;
    transform: translateX(-50%);
    padding: 4px 12px;
    border-radius: 999px;
    background: var(--surface);
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 650;
    pointer-events: none;
    white-space: nowrap;
  }
  .reset {
    position: absolute;
    left: 12px;
    top: 12px;
    background: var(--surface);
  }
  .panel {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    gap: 12px;
    min-height: 0;
    padding: 18px;
    border-radius: 18px;
  }
  header {
    display: grid;
    gap: 2px;
  }
  .name {
    font-size: 20px;
  }
  .parts {
    display: grid;
    align-content: start;
    gap: 4px;
    overflow-y: auto;
    margin: 0 -6px;
    padding: 0 6px;
  }
  .part {
    border-radius: 12px;
    transition: background 160ms var(--ease);
  }
  .part.open {
    background: color-mix(in srgb, var(--surface) 70%, transparent);
  }
  .row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border-radius: 12px;
    text-align: left;
  }
  .row:hover {
    background: color-mix(in srgb, var(--surface) 50%, transparent);
  }
  .label {
    font-weight: 700;
  }
  .value {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chip {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 8px;
    background: var(--c);
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.12);
  }
  .chip.made {
    background: var(--surface);
    color: var(--text-muted);
  }
  .part.open .chip {
    box-shadow:
      inset 0 0 0 1px rgba(0, 0, 0, 0.12),
      0 0 0 2px var(--surface),
      0 0 0 4px var(--accent, var(--build));
  }
  .pick {
    padding: 0 10px 10px 46px;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  /* Narrow screens: the item above, its parts below. */
  @media (max-width: 720px) {
    .studio {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(180px, 40vh) minmax(0, 1fr);
    }
    .pick {
      padding-left: 10px;
    }
  }
</style>
