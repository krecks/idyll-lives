<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { LandTool } from '../../game/landBrush';
  import Icon from '../Icon.svelte';
  import { groupOf, hasHeight, hasStrength, HEIGHTS, LAND_GROUPS, sizeOf, sizeRange, toolInfo, type BrushState } from './landTools';

  /**
   * The land brushes (step 1's Shape panel, Build mode's Land tab): the tools in four groups, the
   * brush's size and strength (a mountain's height instead), undo, redo and resetting the land, and
   * one line of what the tool does (or why the last stroke stopped). Down the sidebar in step 1
   * (`column`), across the dock in the game (`row`).
   */
  let {
    brush = $bindable(),
    canUndo = false,
    canRedo = false,
    canReset = false,
    hint = null,
    layout = 'column',
    disabled = false,
    price,
    onundo,
    onredo,
    onreset,
    extra,
  }: {
    brush: BrushState;
    canUndo?: boolean;
    canRedo?: boolean;
    /** The land differs from how it was first surveyed. */
    canReset?: boolean;
    /** Why the last stroke stopped or left something out (null: the tool's own line). */
    hint?: string | null;
    layout?: 'column' | 'row';
    disabled?: boolean;
    /** What a tool costs (Living), for its tooltip. */
    price?: (tool: LandTool) => string;
    /** Undo and redo: none in the game (the dock's own buttons are the land's while the tab is up). */
    onundo?: () => void;
    onredo?: () => void;
    onreset: () => void;
    /** More, after the buttons (step 1: laying the town again). */
    extra?: Snippet;
  } = $props();

  const info = $derived(toolInfo(brush.tool));
  const range = $derived(sizeRange(brush.tool));
  const size = $derived(sizeOf(brush));
  /** "Reset the land" asks first, in place. */
  let confirming = $state(false);

  function pick(tool: LandTool) {
    brush = { ...brush, tool };
  }

  function setSize(v: number) {
    brush = { ...brush, sizes: { ...brush.sizes, [groupOf(brush.tool)]: v } };
  }
</script>

{#snippet glyph(t: LandTool)}
  <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" stroke-linecap="round">
    {#if t === 'raise'}
      <path d="M2.5 20h19M4 20c2.5-6 13.5-6 16 0M12 13V4M8.8 7.2 12 4l3.2 3.2" />
    {:else if t === 'lower'}
      <path d="M2.5 11H6c1.5 7 10.5 7 12 0h3.5M12 3v8M8.8 7.8 12 11l3.2-3.2" />
    {:else if t === 'smooth'}
      <path d="M2.5 9c2-3 4-3 6 0s4 3 6 0 4-3 7 0M2.5 17c2-1.2 4-1.2 6 0s4 1.2 6 0 4-1.2 7 0" />
    {:else if t === 'flatten'}
      <path d="M2.5 18h19M8 4v8M5 9l3 3 3-3M16 4v8M13 9l3 3 3-3" />
    {:else if t === 'lake'}
      <path d="M3 14c0-3.5 4-5.5 9-5.5s9 2 9 5.5-4 5.5-9 5.5-9-2-9-5.5zM8.5 14h4M13.5 16.5h2.5M12 2.5c1.6 2 2.5 3.3 2.5 4.4a2.5 2.5 0 0 1-5 0c0-1.1.9-2.4 2.5-4.4z" />
    {:else if t === 'river'}
      <path d="M6 2.5c4 4-3 7 1 11s1 8 1 8M15 2.5c4 4-3 7 1 11s1 8 1 8" />
    {:else if t === 'dry'}
      <path d="M12 3.5c3.2 4 5 6.8 5 9.5a5 5 0 0 1-10 0c0-2.7 1.8-5.5 5-9.5zM4 21 20 5" />
    {:else if t === 'woods'}
      <path d="M8 21v-4M8 17H3.5L8 8l4.5 9zM16.5 21v-3M16.5 18h-4l4-8.5 4 8.5zM2.5 21h19" />
    {:else if t === 'meadow'}
      <path d="M4 21c0-3 .8-5 2-6.5M7 21c0-2.4-.3-4.4-1.4-6M12 21v-6.5M12 11.5a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM17 21c0-3 1-4.6 2.5-5.6M20 21c-.3-2-1-3.3-2.2-4.2M2.5 21h19" />
    {:else if t === 'fields'}
      <path d="M2.5 20 6.5 7h15l-4 13zM7.5 20l3.5-13M12.5 20 16 7M4.6 13.5h14.8" />
    {:else if t === 'clear'}
      <path d="M7.5 19.5h13M5 16l7.5-10 6 4.5-6.5 9H8zM9.2 10.4l6 4.5" />
    {:else if t === 'peak'}
      <path d="M2.5 20 10 7l3.5 6 2-3L21.5 20zM7.6 11.2l2.4 1.4 2.2-1.6" />
    {:else if t === 'range'}
      <path d="M1.5 20 6 12l2.5 3.5L13 6l3 5.5 1.5-2.5 5 11zM10.6 10.6 13 12l2-1.4" />
    {:else}
      <path d="M2.5 20h19M4.5 20c2.5-4 5-6 7.5-6s5 2 7.5 6M12 3v7M8.8 6.8 12 10l3.2-3.2" />
    {/if}
  </svg>
{/snippet}

<div class="palette {layout}" class:disabled>
  <div class="groups">
    {#each LAND_GROUPS as g (g.id)}
      <section class="group {g.id}" aria-label={g.label}>
        <h3 class="eyebrow">{g.label}</h3>
        <div class="tools" role="radiogroup" aria-label="{g.label} tools">
          {#each g.tools as t (t.id)}
            <button
              class="tool"
              role="radio"
              aria-checked={brush.tool === t.id}
              class:on={brush.tool === t.id}
              {disabled}
              title="{t.label}{price ? ` · ${price(t.id)}` : ''}: {t.does}"
              onclick={() => pick(t.id)}
            >
              <span class="badge">{@render glyph(t.id)}</span>
              <span class="label">{t.label}</span>
            </button>
          {/each}
        </div>
      </section>
    {/each}
  </div>

  <div class="side">
    <div class="sliders">
      <label class="slider">
        <span class="name">Size</span>
        <input type="range" min={range.min} max={range.max} step="1" value={size} {disabled} oninput={(e) => setSize(Number(e.currentTarget.value))} />
        <span class="value tabular">{size} m</span>
      </label>
      {#if hasHeight(brush.tool)}
        <label class="slider">
          <span class="name">Height</span>
          <input type="range" min={HEIGHTS.min} max={HEIGHTS.max} step="1" value={brush.height} {disabled} oninput={(e) => (brush = { ...brush, height: Number(e.currentTarget.value) })} />
          <span class="value tabular">{brush.height} m</span>
        </label>
      {:else}
        <label class="slider" class:off={!hasStrength(brush.tool)}>
          <span class="name">Strength</span>
          <input
            type="range"
            min="0.05"
            max="1"
            step="0.05"
            value={brush.strength}
            disabled={disabled || !hasStrength(brush.tool)}
            oninput={(e) => (brush = { ...brush, strength: Number(e.currentTarget.value) })}
          />
          <span class="value tabular">{hasStrength(brush.tool) ? `${Math.round(brush.strength * 100)}%` : '—'}</span>
        </label>
      {/if}
    </div>

    {#key hint ?? brush.tool}
      <p class="hint" class:warn={!!hint} aria-live="polite"><b>{info.label}</b> {hint ?? info.does}</p>
    {/key}

    <div class="actions">
      {#if onundo}
        <button class="btn ghost small" disabled={disabled || !canUndo} title="Undo the last stroke (Ctrl+Z / ⌘Z)" onclick={onundo}><Icon name="icon.ui.undo" size={14} /> Undo</button>
      {/if}
      {#if onredo}
        <button class="btn ghost small" disabled={disabled || !canRedo} title="Redo (Ctrl+Shift+Z / ⌘⇧Z)" onclick={onredo}><span class="mirror"><Icon name="icon.ui.undo" size={14} /></span> Redo</button>
      {/if}
      {#if confirming}
        <span class="confirm" role="group" aria-label="Reset the land?">
          <button class="btn small danger" onclick={() => ((confirming = false), onreset())}>Reset all</button>
          <button class="btn ghost small" onclick={() => (confirming = false)}>Keep</button>
        </span>
      {:else}
        <button class="btn ghost small reset" disabled={disabled || !canReset} title="Put the land back as it was" onclick={() => (confirming = true)}>Reset the land</button>
      {/if}
    </div>
    {@render extra?.()}
  </div>
</div>

<style>
  .palette {
    --land: #3f8f5a;
    --land-soft: rgba(63, 143, 90, 0.13);
    display: flex;
    gap: 12px;
  }
  .palette.column {
    flex-direction: column;
  }
  .palette.disabled .groups {
    opacity: 0.55;
  }
  .groups {
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: opacity var(--fast) var(--ease);
  }
  .group {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .group h3 {
    margin: 0;
    font-size: 10.5px;
  }
  /* Each group's own colour, on its badges. */
  .group.water {
    --tone: #2f86c4;
    --tone-soft: rgba(47, 134, 196, 0.13);
  }
  .group.cover {
    --tone: #4f9a3a;
    --tone-soft: rgba(79, 154, 58, 0.14);
  }
  .group.shape {
    --tone: #b07a2e;
    --tone-soft: rgba(176, 122, 46, 0.14);
  }
  .group.mountains {
    --tone: #6c6fa8;
    --tone-soft: rgba(108, 111, 168, 0.15);
  }
  .tools {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 3px;
  }
  .tool {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    min-width: 0;
    padding: 5px 2px 4px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 650;
    transition:
      background var(--fast) var(--ease),
      color var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
  .tool:hover:not(:disabled) {
    color: var(--text);
    background: rgba(255, 255, 255, 0.7);
  }
  .tool:disabled {
    cursor: default;
  }
  .badge {
    display: grid;
    place-items: center;
    width: 36px;
    height: 30px;
    border-radius: 9px;
    background: var(--tone-soft);
    color: var(--tone);
    transition:
      background var(--fast) var(--ease),
      color var(--fast) var(--ease);
  }
  .tool.on {
    background: #fff;
    color: var(--tone);
    border-color: color-mix(in srgb, var(--tone) 35%, transparent);
    box-shadow: 0 0 0 3px var(--tone-soft);
  }
  .tool.on .badge {
    background: var(--tone);
    color: #fff;
  }
  .label {
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 9px;
    min-width: 0;
  }
  .sliders {
    display: grid;
    gap: 6px;
  }
  .slider {
    display: grid;
    grid-template-columns: 62px minmax(0, 1fr) 46px;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }
  .slider.off {
    opacity: 0.5;
  }
  .slider input {
    margin: 0;
    accent-color: var(--land);
  }
  .value {
    text-align: right;
    color: var(--text-muted);
    font-size: 11.5px;
  }
  /* (Two lines kept for it: the buttons under it stay put from tool to tool.) */
  .hint {
    margin: 0;
    min-height: 2lh;
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-muted);
    animation: step 260ms var(--ease);
  }
  .hint b {
    margin-right: 3px;
    color: var(--text-strong);
  }
  .hint.warn {
    color: #a3552c;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
  }
  .btn.small {
    height: 30px;
    padding: 0 10px;
    gap: 5px;
    font-size: 12.5px;
  }
  .reset,
  .confirm {
    margin-left: auto;
  }
  .confirm {
    display: inline-flex;
    gap: 4px;
  }
  .danger {
    background: var(--bad);
    color: #fff;
  }
  .mirror {
    display: inline-flex;
    transform: scaleX(-1);
  }

  /* Across the dock: the groups side by side (hairlines between), the sliders and the rest beside them. */
  .palette.row {
    align-items: flex-start;
  }
  .row .groups {
    flex: none;
    flex-direction: row;
    gap: 0;
  }
  .row .group {
    padding: 0 9px;
    border-left: 1px solid var(--hairline);
  }
  .row .group:first-child {
    padding-left: 0;
    border-left: 0;
  }
  .row .tools {
    display: flex;
  }
  .row .tool {
    width: 52px;
  }
  .row .side {
    flex: 1 1 260px;
    padding-left: 10px;
    border-left: 1px solid var(--hairline);
  }
  .row .hint {
    min-height: 0;
  }
  @keyframes step {
    from {
      opacity: 0;
      transform: translateX(-4px);
    }
  }
</style>
