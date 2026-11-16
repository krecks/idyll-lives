<script lang="ts">
  import type { LandTool } from '../../game/landBrush';
  import { TOOL_KIND } from '../../game/landBrush';
  import { money } from '../format';
  import LandPalette from '../land/LandPalette.svelte';
  import { services } from '../services';
  import { game } from '../state.svelte';
  import BuildDock from './BuildDock.svelte';

  /**
   * Build mode's Land tab: the land brushes (`LandPalette`) across the dock, over the whole world
   * (the camera roams; built land is left as it is). What a stroke costs shows at the cursor while
   * it is drawn (`BuyFx`); the dock's Undo and Redo take strokes back and put them again.
   */
  const prices = $derived(game.catalog?.build.land ?? null);

  /** A tool's price, for its tooltip: free in Creative, else its rate (a cubic metre of earth moved, a square metre painted). */
  function price(tool: LandTool): string {
    if (game.creative) return 'Free';
    if (!prices) return '';
    const kind = TOOL_KIND[tool];
    if (kind === 'earth') return `${money(prices.earth)} a m³ moved`;
    if (kind === 'mountain') return `${cents(prices.mountain)} a m³ (a peak a few thousand)`;
    if (kind === 'water') return `${money(prices.water)} a m² of water`;
    const each = tool === 'woods' ? prices.woods : tool === 'meadow' ? prices.meadow : tool === 'fields' ? prices.fields : prices.clear;
    return `${cents(each)} a m²`;
  }
  const cents = (v: number) => (v >= 1 && Number.isInteger(v) ? money(v) : `$${v.toFixed(2).replace(/0$/, '')}`);
</script>

<BuildDock label="Land" tipId="land" style="--build: #3f8f5a; --build-soft: rgba(63, 143, 90, 0.14)">
  {#snippet tip()}
    <b>Shape the land.</b> Pick a brush and drag over the world to raise hills, dig ponds and rivers, plant woods or raise mountains. Lots and roads are left as they are.
    {game.creative ? 'In Creative it is free.' : 'Each stroke is paid for; undo gives the money back.'} Right-drag turns the camera; <kbd>Esc</kbd> or right-click cancels a stroke.
  {/snippet}

  {#snippet head()}
    <p class="hint">
      <b>Land</b>
      {game.creative ? 'Free in Creative' : 'Paid by the stroke · undo refunds it'}<span class="muted">{' · '}built land is left as it is · <kbd>Esc</kbd> or right-click cancels</span>
    </p>
  {/snippet}

  {#snippet body()}
    {#if game.land.surveying || !game.land.ready}
      <p class="surveying" role="status"><span class="spinner"></span> Surveying the land…</p>
    {:else}
      <LandPalette
        bind:brush={game.landBrush}
        layout="row"
        canReset={game.land.changed}
        hint={game.land.hint}
        disabled={game.land.busy}
        {price}
        onreset={() => services.controls.landReset()}
      />
    {/if}
  {/snippet}
</BuildDock>

{#if game.landGhost}
  <svg class="ghost" aria-hidden="true"><path d={game.landGhost} /></svg>
{/if}

<style>
  .hint {
    margin: 0;
    min-width: 0;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hint b {
    margin-right: 4px;
    color: var(--build);
  }
  .muted {
    color: var(--text-muted);
  }
  kbd {
    padding: 0 5px;
    border-radius: 4px;
    background: var(--hairline);
    font-family: inherit;
    font-size: 11px;
  }
  .surveying {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 14px 4px;
    font-size: 13px;
    font-weight: 600;
  }
  /* A mountain's ghost, in screen pixels (the HUD's scale undone). */
  .ghost {
    position: fixed;
    inset: 0;
    width: 100%;
    height: 100%;
    zoom: calc(1 / var(--ui-scale));
    pointer-events: none;
    overflow: visible;
  }
  .ghost path {
    fill: none;
    stroke: rgba(255, 255, 255, 0.85);
    stroke-width: 1.4;
    stroke-linejoin: round;
    filter: drop-shadow(0 0 1.5px rgba(40, 46, 90, 0.7));
  }
</style>
