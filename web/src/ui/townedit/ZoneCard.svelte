<script lang="ts">
  /**
   * The zone brush's card: the five uses as chips in their colours (`uses.ts`), Erase, the brush's
   * size and "Beside streets only" (`zoneChoice`). Under them, what the zones would build, about.
   */
  import Segmented from '../kit/Segmented.svelte';
  import { USE_COLORS, USE_KINDS, USE_LABELS } from '../../game/uses';
  import { summaryLine, usesSummary } from './layoutApi';
  import type { TownEditor } from './tools';
  import { ZONE_SIZES, zoneChoice } from './zoneChoice.svelte';

  const { editor }: { editor: TownEditor } = $props();

  const SIZES = ZONE_SIZES.map((value) => ({ value: String(value), label: `${value} m` }));
  const line = $derived((void editor.version, summaryLine(usesSummary(editor.draft))));
</script>

<div class="zone-card">
  <div class="choices" role="radiogroup" aria-label="What to zone the land for">
    {#each USE_KINDS as k (k)}
      <button role="radio" class="chip" aria-checked={zoneChoice.kind === k} class:on={zoneChoice.kind === k} data-use={k} style:--swatch={USE_COLORS[k]} onclick={() => (zoneChoice.kind = k)}>
        <span class="swatch" aria-hidden="true"></span>{USE_LABELS[k]}
      </button>
    {/each}
    <button role="radio" class="chip erase" aria-checked={zoneChoice.kind === null} class:on={zoneChoice.kind === null} data-use="erase" onclick={() => (zoneChoice.kind = null)}>Erase</button>
  </div>
  <div class="option">
    <span class="eyebrow sub">Brush size</span>
    <Segmented fill label="Brush size" value={String(zoneChoice.size)} options={SIZES} onchange={(v) => (zoneChoice.size = Number(v) as (typeof ZONE_SIZES)[number])} />
  </div>
  <label class="beside">
    <input type="checkbox" bind:checked={zoneChoice.beside} />
    <span>Beside streets only <span class="sub">(within 40 m of a road)</span></span>
  </label>
  <p class="hint">Paint where homes, shops, parks and public places go; Build puts them along the streets. Zones far from a street are hatched: nothing is built there until a street reaches them.</p>
  <p class="hint summary" aria-live="polite">{line}</p>
</div>

<style>
  .zone-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    color: var(--text);
    font-size: 12px;
    font-weight: 600;
  }
  .chip.on {
    background: #fff;
    color: var(--text-strong);
    box-shadow:
      0 0 0 2px var(--swatch, var(--accent)),
      0 0 0 5px color-mix(in srgb, var(--swatch, var(--accent)) 25%, transparent);
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    background: var(--swatch);
  }
  .erase.on {
    --swatch: #8a93a6;
  }
  .beside {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 550;
  }
  .beside .sub {
    color: var(--text-muted);
    font-weight: 500;
  }
  .hint {
    margin: 0;
  }
  .summary {
    color: var(--text);
    font-weight: 600;
  }
</style>
