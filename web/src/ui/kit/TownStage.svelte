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

  import type { StageTool } from './stageTool';
  export type { StageTool };
  import { menuScene, type Frame } from '../../game/menuScene';
  import { OVERVIEW_NEAREST } from '../../render/shared/overviewCamera';

  /** The stages up, each with how near its zoom comes in: the one put up last has its way (a screen's comes before the last one's goes). */
  const claims: { metres: number }[] = [];

  /** `metres` as the zoom's nearest while the stage is up; the function given back lets go of it. */
  function claimNearest(metres: number): () => void {
    const claim = { metres };
    claims.push(claim);
    menuScene.setNearest(metres);
    return () => {
      claims.splice(claims.indexOf(claim), 1);
      menuScene.setNearest(claims.at(-1)?.metres ?? OVERVIEW_NEAREST);
    };
  }
</script>

<script lang="ts">
  /**
   * A transparent stage over the live 3D town (drawn by the renderer behind the page). Frames the
   * camera into this box, turns pointer input into lot hover / clicks and the camera's moves, and
   * pins a tag above every lot. The camera itself is driven by `menuScene`.
   *
   * The camera, in every step: the right button turns it (across) and tilts it (up and down); the
   * middle button pans it, as does the left with Space held; the wheel zooms towards the pointer;
   * held W A S D or the arrows pan it, Q and E turn it, R and F tilt it, and Home frames the shot
   * again. Keys are let be while typing, with a modifier held, or when the tool in hand takes them
   * (the editor's own handler marks them done: they are looked at a frame later, `heldKeys`).
   *
   * With a `tool` in hand the left button is the tool's (a right press mid-stroke drops the stroke);
   * lots aren't lit up or tagged meanwhile. With none, the left button picks a lot, and a drag with
   * it pans. `ghost`: SVG path data drawn over the town (a mountain's ghost, screen pixels).
   *
   * `shot`: what the camera frames with no lot in focus: the main town (`overview`), the whole world
   * (`world`), or one of its areas (`{ area }`: the main town is 0, then the villages). `nearest`: how
   * near (m) the player's zoom comes in (`OVERVIEW_CLOSE` in the steps that shape the town).
   */
  import { onMount } from 'svelte';

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
    shot = 'overview',
    nearest = OVERVIEW_NEAREST,
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
    shot?: 'overview' | 'world' | { area: number };
    /** How near (m from what it looks at) the player's zoom comes in. */
    nearest?: number;
  } = $props();

  let stage: HTMLDivElement;
  let tags: HTMLDivElement;
  let frame = $state<Frame | null>(null);
  let dragging = false;
  /** The left button down with no tool in hand: a click on a lot, or (once it moves) a pan. */
  let down: { x: number; y: number; moved: boolean } | null = null;
  let cursor = $state('default');
  /** A drag: the tool's stroke, the camera turned and tilted, or panned. */
  let held: { kind: 'stroke' | 'orbit' | 'pan'; x: number; y: number } | null = null;
  /** Space is held (with a tool in hand, the left button pans). */
  let space = $state(false);

  /** How far (radians) a CSS px of a right-button drag turns the camera across, and tilts it up and down. */
  const TURN_PX = 0.006;
  const TILT_PX = 0.005;
  /** Held keys: the pan (a share of the camera's distance a second), the turn and the tilt (radians a second). */
  const PAN_RATE = 0.9;
  const TURN_RATE = 1.4;
  const TILT_RATE = 0.9;
  /** The camera's keys (by `KeyboardEvent.code`: where they are, whatever the layout): pan right and forward, turn, tilt. */
  const KEYS: Record<string, { right?: number; forward?: number; turn?: number; tilt?: number }> = {
    KeyW: { forward: 1 },
    ArrowUp: { forward: 1 },
    KeyS: { forward: -1 },
    ArrowDown: { forward: -1 },
    KeyA: { right: -1 },
    ArrowLeft: { right: -1 },
    KeyD: { right: 1 },
    ArrowRight: { right: 1 },
    KeyQ: { turn: 1 },
    KeyE: { turn: -1 },
    KeyR: { tilt: -1 },
    KeyF: { tilt: 1 },
  };

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

  // How near the zoom comes in, while this stage is up (the stage of the screen shown last wins).
  $effect(() => claimNearest(nearest));

  function typing(target: EventTarget | null): boolean {
    return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement || (target instanceof HTMLElement && target.isContentEditable);
  }

  // ---- The camera's keys ----------------------------------------------------------------------

  /** The camera's keys held, by code, with the event that last pressed them (a key the tool took is let go). */
  const heldKeys = new Map<string, KeyboardEvent>();
  /** Home pressed: the shot framed again on the next frame (unless something else took the key). */
  let home: KeyboardEvent | null = null;
  let loop = 0;
  let last = 0;

  /**
   * A frame of the held keys: whatever another handler took (the tool's `key`, a radio group's arrows) is let go,
   * the rest move the camera by the time since the last frame.
   */
  function keysFrame(now: number) {
    const dt = Math.min(0.1, Math.max(0, (now - last) / 1000));
    last = now;
    if (home && !home.defaultPrevented) menuScene.reframe();
    home = null;
    let [right, forward, turn, tilt] = [0, 0, 0, 0];
    for (const [code, e] of heldKeys) {
      if (e.defaultPrevented) {
        heldKeys.delete(code);
        continue;
      }
      const k = KEYS[code];
      right += k.right ?? 0;
      forward += k.forward ?? 0;
      turn += k.turn ?? 0;
      tilt += k.tilt ?? 0;
    }
    const pan = PAN_RATE * menuScene.distance * dt;
    if (right || forward) menuScene.panView(Math.sign(right) * pan, Math.sign(forward) * pan);
    if (turn || tilt) menuScene.nudge(Math.sign(turn) * TURN_RATE * dt, 1, Math.sign(tilt) * TILT_RATE * dt);
    loop = heldKeys.size ? requestAnimationFrame(keysFrame) : 0;
  }

  function startKeys() {
    if (loop) return;
    last = performance.now();
    loop = requestAnimationFrame(keysFrame);
  }

  function stopKeys() {
    heldKeys.clear();
    home = null;
    if (loop) cancelAnimationFrame(loop);
    loop = 0;
  }

  /** A camera key pressed (not while typing, nor with a modifier): taken in from the next frame. */
  function cameraKey(e: KeyboardEvent): void {
    if (e.ctrlKey || e.metaKey || e.altKey || e.shiftKey || typing(e.target)) return;
    if (e.code === 'Home') {
      home = e;
      startKeys();
    } else if (KEYS[e.code]) {
      heldKeys.set(e.code, e);
      startKeys();
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (tool && e.key === 'Escape' && held?.kind === 'stroke') {
      e.preventDefault();
      tool.cancel();
      held = null;
      return;
    }
    if (e.key === ' ') {
      if (!tool || typing(e.target) || e.target instanceof HTMLButtonElement) return;
      e.preventDefault();
      space = true;
      if (!held) cursor = 'grab';
      return;
    }
    cameraKey(e);
  }

  function onkeyup(e: KeyboardEvent) {
    heldKeys.delete(e.code);
    if (e.key !== ' ') return;
    space = false;
    if (!held) cursor = tool ? 'crosshair' : 'default';
  }

  function onblur() {
    space = false;
    stopKeys();
  }

  onMount(() => stopKeys);

  // ---- The pointer ----------------------------------------------------------------------------

  /** A drag of the camera begun (`orbit`: turned and tilted; `pan`: the land follows the pointer). */
  function hold(kind: 'orbit' | 'pan', e: PointerEvent): void {
    held = { kind, x: e.clientX, y: e.clientY };
    cursor = 'grabbing';
    stage.setPointerCapture(e.pointerId);
  }

  /** The camera's drag followed to the pointer's new place. */
  function dragTo(e: PointerEvent): void {
    if (!held) return;
    const [dx, dy] = [e.clientX - held.x, e.clientY - held.y];
    if (held.kind === 'orbit') menuScene.nudge(-dx * TURN_PX, 1, -dy * TILT_PX);
    else if (held.kind === 'pan') menuScene.drag(held.x, held.y, e.clientX, e.clientY);
    held.x = e.clientX;
    held.y = e.clientY;
  }

  function letGo(e: PointerEvent): void {
    held = null;
    if (stage.hasPointerCapture(e.pointerId)) stage.releasePointerCapture(e.pointerId);
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
    if (kind !== 'stroke') {
      t.leave();
      hold(kind, e);
      return;
    }
    if (!t.down(e.clientX, e.clientY)) return;
    held = { kind, x: e.clientX, y: e.clientY };
    stage.setPointerCapture(e.pointerId);
  }

  function toolMove(e: PointerEvent): void {
    const t = tool!;
    if (!held) {
      t.hover(e.clientX, e.clientY);
      cursor = space ? 'grab' : 'crosshair';
      return;
    }
    if (held.kind !== 'stroke') return dragTo(e);
    t.move(e.clientX, e.clientY);
    held.x = e.clientX;
    held.y = e.clientY;
  }

  function toolUp(e: PointerEvent): void {
    if (!held) return;
    if (held.kind === 'stroke') tool!.up();
    letGo(e);
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

  // Camera: the lot in focus, or the town, the world or one of its areas, framed into the stage.
  $effect(() => {
    if (!frame) return;
    if (focus !== null) menuScene.shoot({ kind: 'lot', plot: focus, frame });
    else if (shot === 'world') menuScene.shoot({ kind: 'world', frame });
    else if (typeof shot === 'object') menuScene.shoot({ kind: 'area', area: shot.area, frame });
    else menuScene.shoot({ kind: 'overview', frame });
  });

  function pointAt(e: PointerEvent): number | null {
    return menuScene.plotAt(e.clientX, e.clientY);
  }

  function onpointerdown(e: PointerEvent) {
    if (tool) return toolDown(e);
    if (held || down) return;
    if (e.button === 1 || e.button === 2) {
      if (e.button === 1) e.preventDefault();
      hold(e.button === 1 ? 'pan' : 'orbit', e);
      return;
    }
    if (e.button !== 0) return;
    down = { x: e.clientX, y: e.clientY, moved: false };
    stage.setPointerCapture(e.pointerId);
  }

  function onpointermove(e: PointerEvent) {
    if (tool) return toolMove(e);
    if (held) return dragTo(e);
    if (down) {
      if (!down.moved && Math.abs(e.clientX - down.x) + Math.abs(e.clientY - down.y) > 5) down.moved = dragging = true;
      if (down.moved) {
        // (The land under the pointer follows it, from where the button went down.)
        menuScene.drag(down.x, down.y, e.clientX, e.clientY);
        down.x = e.clientX;
        down.y = e.clientY;
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
    if (held) {
      letGo(e);
      cursor = 'default';
      return;
    }
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
    menuScene.zoomAt(e.clientX, e.clientY, Math.exp(-e.deltaY * 0.0015));
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

<svelte:window {onkeydown} {onkeyup} {onblur} />

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
    down = null;
    dragging = false;
    letGo(e);
  }}
  oncontextmenu={(e) => e.preventDefault()}
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
