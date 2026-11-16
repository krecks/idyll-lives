<script lang="ts" module>
  export interface LotLabel {
    plot: number;
    title: string;
    sub?: string;
    /** Look of the tag: a household, a house for sale, a public place (park or shop), the player's new home. */
    kind: 'home' | 'sale' | 'park' | 'mine';
  }

  /** Which kinds' tags win room first, unless a screen says otherwise (`order`): the player's, households', houses for sale, places. */
  export const TAG_ORDER: readonly LotLabel['kind'][] = ['mine', 'home', 'sale', 'park'];

  /** A tool in hand on the town (step 1's land brushes): the left button is its, at screen points (CSS pixels). */
  export interface StageTool {
    /** Pressed at (`x`, `y`): true if it took the press (a stroke began). */
    down(x: number, y: number): boolean;
    move(x: number, y: number): void;
    up(): void;
    /** The pointer over the town with no button down (the cursor follows it). */
    hover(x: number, y: number): void;
    leave(): void;
    /** The stroke under way dropped (right-click, Escape). */
    cancel(): void;
  }
</script>

<script lang="ts">
  /**
   * A transparent stage over the live 3D town (drawn by the renderer behind the page). Frames the
   * camera into this box, turns pointer input into lot hover / clicks, a drag to look around and
   * the wheel to zoom, and pins a tag above every lot. The camera itself is driven by `menuScene`.
   *
   * With a `tool` in hand the left button is the tool's, the right button turns the camera, the
   * middle button (or the left with Space held) pans it, the wheel zooms; lots aren't lit up or
   * tagged meanwhile. `ghost`: SVG path data drawn over the town (a mountain's ghost, screen pixels).
   */
  import { onMount } from 'svelte';
  import { menuScene, type Frame } from '../../game/menuScene';

  let {
    labels,
    hover = null,
    selected = null,
    focus = null,
    selectable = () => true,
    order = TAG_ORDER,
    onhover,
    onselect,
    tool = null,
    ghost = '',
  }: {
    labels: LotLabel[];
    /** Which kinds' tags win room first (`tagLayout.ts`); households' tags are also only so many at once. */
    order?: readonly LotLabel['kind'][];
    hover?: number | null;
    selected?: number | null;
    /** Lot the camera glides to (null: the whole town). */
    focus?: number | null;
    selectable?: (plot: number) => boolean;
    onhover?: (plot: number | null) => void;
    /** A lot (or empty ground: null) was clicked. */
    onselect?: (plot: number | null) => void;
    tool?: StageTool | null;
    ghost?: string;
  } = $props();

  let stage: HTMLDivElement;
  let tags: HTMLDivElement;
  let frame = $state<Frame | null>(null);
  let dragging = false;
  let down: { x: number; y: number; moved: boolean } | null = null;
  let cursor = $state('default');
  /** A drag with a tool in hand: the tool's stroke, the camera turned, or panned. */
  let held: { kind: 'stroke' | 'orbit' | 'pan'; x: number; y: number } | null = null;
  /** Space is held (with a tool in hand, the left button pans). */
  let space = $state(false);

  // A tool put down mid-stroke: its stroke dropped. (Lots lit up before it are let go.)
  $effect(() => {
    if (!tool) return;
    onhover?.(null);
    const t = tool;
    return () => {
      if (held?.kind === 'stroke') t.cancel();
      held = null;
      t.leave();
    };
  });

  function typing(target: EventTarget | null): boolean {
    return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement || (target instanceof HTMLElement && target.isContentEditable);
  }

  function onkeydown(e: KeyboardEvent) {
    if (!tool) return;
    if (e.key === 'Escape' && held?.kind === 'stroke') {
      e.preventDefault();
      tool.cancel();
      held = null;
      return;
    }
    if (e.key !== ' ' || typing(e.target) || e.target instanceof HTMLButtonElement) return;
    e.preventDefault();
    space = true;
    if (!held) cursor = 'grab';
  }

  function onkeyup(e: KeyboardEvent) {
    if (e.key !== ' ') return;
    space = false;
    if (!held) cursor = tool ? 'crosshair' : 'default';
  }

  /** Pans the camera by the ground between two screen points (as if the land were dragged along), or by a share of the view where the sky is. */
  function panBy(x0: number, y0: number, x1: number, y1: number) {
    const a = menuScene.landAt(x0, y0);
    const b = menuScene.landAt(x1, y1);
    if (a && b) menuScene.pan(a.x - b.x, a.z - b.z);
  }

  function toolDown(e: PointerEvent): void {
    const t = tool!;
    if (e.button === 2 && held?.kind === 'stroke') {
      t.cancel();
      held = null;
      return;
    }
    if (held) return;
    const kind = e.button === 0 && !space ? 'stroke' : e.button === 1 || e.button === 0 ? 'pan' : e.button === 2 ? 'orbit' : null;
    if (!kind) return;
    if (e.button === 1) e.preventDefault();
    if (kind === 'stroke' && !t.down(e.clientX, e.clientY)) return;
    held = { kind, x: e.clientX, y: e.clientY };
    if (kind !== 'stroke') {
      t.leave();
      cursor = 'grabbing';
    }
    stage.setPointerCapture(e.pointerId);
  }

  function toolMove(e: PointerEvent): void {
    const t = tool!;
    if (!held) {
      t.hover(e.clientX, e.clientY);
      cursor = space ? 'grab' : 'crosshair';
      return;
    }
    if (held.kind === 'stroke') t.move(e.clientX, e.clientY);
    else if (held.kind === 'orbit') menuScene.nudge(-e.movementX * 0.006, 1);
    else panBy(held.x, held.y, e.clientX, e.clientY);
    held.x = e.clientX;
    held.y = e.clientY;
  }

  function toolUp(e: PointerEvent): void {
    if (!held) return;
    if (held.kind === 'stroke') tool!.up();
    held = null;
    if (stage.hasPointerCapture(e.pointerId)) stage.releasePointerCapture(e.pointerId);
    cursor = space ? 'grab' : 'crosshair';
  }

  onMount(() => {
    const measure = () => {
      const r = stage.getBoundingClientRect();
      frame = { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(stage);
    window.addEventListener('resize', measure);
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', measure);
    };
  });

  // Camera: the lot in focus, or the whole town, framed into the stage.
  $effect(() => {
    if (!frame) return;
    menuScene.shoot(focus === null ? { kind: 'overview', frame } : { kind: 'lot', plot: focus, frame });
  });

  function pointAt(e: PointerEvent): number | null {
    return menuScene.plotAt(e.clientX, e.clientY);
  }

  function onpointerdown(e: PointerEvent) {
    if (tool) return toolDown(e);
    if (e.button !== 0) return;
    down = { x: e.clientX, y: e.clientY, moved: false };
    stage.setPointerCapture(e.pointerId);
  }

  function onpointermove(e: PointerEvent) {
    if (tool) return toolMove(e);
    if (down) {
      const dx = e.clientX - down.x;
      if (!down.moved && Math.abs(dx) + Math.abs(e.clientY - down.y) > 5) down.moved = dragging = true;
      if (down.moved) {
        menuScene.nudge(-e.movementX * 0.006, 1);
        cursor = 'grabbing';
        return;
      }
    }
    const plot = pointAt(e);
    if (plot !== hover) onhover?.(plot);
    cursor = plot !== null && selectable(plot) ? 'pointer' : 'grab';
  }

  function onpointerup(e: PointerEvent) {
    if (tool) return toolUp(e);
    const click = down && !down.moved;
    down = null;
    dragging = false;
    if (stage.hasPointerCapture(e.pointerId)) stage.releasePointerCapture(e.pointerId);
    if (!click) return;
    const plot = pointAt(e);
    onselect?.(plot !== null && selectable(plot) ? plot : null);
  }

  function onwheel(e: WheelEvent) {
    e.preventDefault();
    menuScene.nudge(0, Math.exp(-e.deltaY * 0.0015));
  }

  /** Svelte action: keeps a tag pinned above its lot, ranked by its kind (`order`). */
  function pin(el: HTMLElement, l: LotLabel) {
    let [plot, kind] = [l.plot, l.kind];
    let detach = menuScene.pin(plot, el, order.indexOf(kind), kind === 'home');
    return {
      // (The labels are made again on every hover: only another lot or kind pins it again.)
      update(next: LotLabel) {
        if (next.plot === plot && next.kind === kind) return;
        [plot, kind] = [next.plot, next.kind];
        detach();
        detach = menuScene.pin(plot, el, order.indexOf(kind), kind === 'home');
      },
      destroy: () => detach(),
    };
  }
</script>

<svelte:window {onkeydown} {onkeyup} onblur={() => (space = false)} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="stage"
  bind:this={stage}
  style:cursor
  {onpointerdown}
  {onpointermove}
  {onpointerup}
  onpointercancel={(e) => {
    if (tool && held?.kind === 'stroke') tool.cancel();
    held = null;
    if (stage.hasPointerCapture(e.pointerId)) stage.releasePointerCapture(e.pointerId);
  }}
  oncontextmenu={(e) => tool && e.preventDefault()}
  onpointerleave={(e) => {
    if (tool) {
      if (!held) tool.leave();
      return;
    }
    // Moving onto a tag is not leaving the lot: only clear the hover when the pointer goes
    // somewhere else (the page beside the stage, another panel).
    if (!dragging && !(e.relatedTarget instanceof Node && tags.contains(e.relatedTarget))) onhover?.(null);
  }}
  {onwheel}
></div>

{#if ghost}
  <svg class="ghost" aria-hidden="true"><path d={ghost} /></svg>
{/if}

<div class="tags" bind:this={tags} aria-label="Lots">
  {#each tool ? [] : labels as l (l.plot)}
    <div class="pin" use:pin={l}>
      <button
        class="tag {l.kind}"
        class:hover={hover === l.plot}
        class:selected={selected === l.plot}
        disabled={!selectable(l.plot)}
        onmouseenter={() => onhover?.(l.plot)}
        onmouseleave={() => onhover?.(null)}
        onclick={() => selectable(l.plot) && onselect?.(l.plot)}
      >
        <span class="dot" aria-hidden="true"></span>
        <span class="text">
          <span class="title">{l.title}</span>
          {#if l.sub}<span class="sub">{l.sub}</span>{/if}
        </span>
      </button>
    </div>
  {/each}
</div>

<style>
  .stage {
    position: relative;
    min-height: 0;
    height: 100%;
    touch-action: none;
  }
  /* A mountain's ghost, in screen pixels (the UI scale undone, as for the tags). */
  .ghost {
    position: fixed;
    inset: 0;
    width: 100%;
    height: 100%;
    zoom: calc(1 / var(--ui-scale));
    pointer-events: none;
    z-index: 1;
    overflow: visible;
  }
  .ghost path {
    fill: none;
    stroke: rgba(255, 255, 255, 0.85);
    stroke-width: 1.4;
    stroke-linejoin: round;
    filter: drop-shadow(0 0 1.5px rgba(40, 46, 90, 0.7));
  }
  /* Tags are positioned in screen pixels: undo the UI scale for the layer, keep it for the tags. */
  .tags {
    position: fixed;
    inset: 0;
    pointer-events: none;
    zoom: calc(1 / var(--ui-scale));
    z-index: 1;
  }
  .pin {
    position: absolute;
    left: 0;
    top: 0;
    will-change: transform;
  }
  .tag {
    position: absolute;
    left: 0;
    bottom: 10px;
    transform: translateX(-50%);
    zoom: var(--ui-scale);
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px 6px 9px;
    border-radius: var(--radius-pill);
    background: var(--tag-bg);
    color: var(--text-strong);
    box-shadow: var(--tag-shadow);
    white-space: nowrap;
    text-align: left;
    transition:
      transform var(--fast) var(--ease),
      background var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
  /* The stem from the tag down to the roof. */
  .tag::after {
    content: '';
    position: absolute;
    left: 50%;
    bottom: -9px;
    width: 2px;
    height: 9px;
    margin-left: -1px;
    border-radius: 1px;
    background: var(--tag-bg);
  }
  .tag:disabled {
    cursor: default;
  }
  /* Lots outside the view: the tag waits at the edge of the stage. */
  .pin:global(.edge) .tag:not(.selected) {
    opacity: 0.86;
    transform: translateX(-50%) scale(0.92);
  }
  .pin:global(.edge) .tag::after {
    display: none;
  }
  .tag:not(:disabled):hover,
  .tag.hover {
    transform: translateX(-50%) translateY(-3px);
    background: #fff;
    box-shadow: var(--tag-shadow-hover);
  }
  .tag.selected {
    background: var(--accent);
    color: #fff;
    transform: translateX(-50%) translateY(-4px) scale(1.06);
    box-shadow: 0 10px 28px rgba(60, 90, 220, 0.45);
  }
  .tag.selected::after {
    background: var(--accent);
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex: none;
    background: var(--tag-home);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--tag-home) 25%, transparent);
  }
  .sale .dot {
    background: var(--tag-sale);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--tag-sale) 30%, transparent);
  }
  .park .dot {
    background: var(--tag-park);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--tag-park) 30%, transparent);
  }
  .mine .dot,
  .selected .dot {
    background: #fff;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.35);
  }
  .text {
    display: flex;
    flex-direction: column;
    line-height: 1.15;
  }
  .title {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .sub {
    font-size: 11px;
    font-weight: 550;
    color: var(--text-muted);
  }
  .selected .sub {
    color: rgba(255, 255, 255, 0.85);
  }
</style>
