<script lang="ts">
  import Icon from './Icon.svelte';
  import { clock } from './format';
  import SimPreview from './kit/SimPreview.svelte';
  import { closeOutside } from './kit/popup';
  import { services } from './services';
  import { game } from './state.svelte';
  import { townMap } from './townMap.svelte';
  import { settings } from '../settings/settings.svelte';

  /**
   * In-game town map: who lives where, who's home, and visiting. It pans (drag) and zooms (the
   * wheel, 1 to 12 times), its labels the same size on screen at any zoom; a big world's land,
   * country roads, trails (dashed), hamlets and outdoor places are on it too, and residents as
   * dots. "Look here" takes the camera to the lot or place picked.
   */
  const plots = $derived(game.plots);
  const meta = $derived(townMap.world?.meta ?? null);
  /** The ground it shows (m), and a margin round it, the same on every side. */
  const width = $derived(townMap.world?.width ?? Math.max(...plots.map((p) => p.x + p.w), 10));
  const depth = $derived(townMap.world?.depth ?? Math.max(...plots.map((p) => p.z + p.d), 10));
  const MARGIN = 6;
  const big = $derived(Math.max(width, depth) > 300);
  const me = $derived(game.selectedSim);
  const homePlot = $derived(game.households.find((h) => h.player)?.plot ?? null);
  let picked = $state<number | null>(null);
  const plot = $derived(picked === null ? null : plots[picked]);
  const household = $derived(plot ? game.households.find((h) => h.plot === plot.id) : undefined);
  const residents = $derived(household ? game.sims.filter((s) => s.household === household.id) : []);
  const info = (id: number) => game.roster.find((r) => r.id === id);
  const homeCount = (plotId: number) => game.sims.filter((s) => s.plot === plotId).length;

  // ---- the view: zoom 1..12 round its middle (m); the map's width on screen (px) ----
  let zoom = $state(1);
  let mid = $state<[number, number] | null>(null);
  let screenW = $state(432);
  let wide = $state(false);
  const fullW = $derived(width + MARGIN * 2);
  const fullD = $derived(depth + MARGIN * 2);
  const viewW = $derived(fullW / zoom);
  const viewD = $derived(fullD / zoom);
  const clampTo = (v: number, size: number, full: number) => Math.max(-MARGIN + size / 2, Math.min(full - MARGIN - size / 2, v));
  const cx = $derived(clampTo(mid?.[0] ?? width / 2, viewW, fullW));
  const cz = $derived(clampTo(mid?.[1] ?? depth / 2, viewD, fullD));
  const x0 = $derived(cx - viewW / 2);
  const z0 = $derived(cz - viewD / 2);
  /** Screen pixels a metre: a size in pixels divided by this stays that size whatever the zoom. */
  const ppm = $derived(screenW / viewW);
  const px = (n: number) => n / ppm;
  /**
   * Where a label's middle goes (m): at `x`, `z`, or as far in from the map's edges as all of `text`
   * at `size` px needs (its width guessed from its length): a place by the edge kept its name cut
   * off there.
   */
  const labelX = (x: number, text: string, size: number) => {
    const half = px(size * text.length * 0.31 + 4);
    return Math.max(-MARGIN + half, Math.min(width + MARGIN - half, x));
  };
  const labelZ = (z: number, size: number) => Math.max(-MARGIN + px(size + 2), Math.min(depth + MARGIN - px(4), z));

  // ---- layers, worked out once per town ----
  const ZONE_ORDER = ['meadow', 'fields', 'hill', 'woods', 'water'];
  const line = (pts: [number, number][]) => pts.map(([x, z]) => `${x},${z}`).join(' ');
  const zones = $derived(
    (meta?.zones ?? []).map((z) => ({ kind: z.kind, d: `M${z.poly.map(([x, zz]) => `${x},${zz}`).join('L')}Z` })).sort((a, b) => ZONE_ORDER.indexOf(a.kind) - ZONE_ORDER.indexOf(b.kind)),
  );
  const roads = $derived((meta?.roads ?? []).map((r) => ({ kind: r.kind ?? 'street', width: r.width, points: line(r.points) })));
  const greens = $derived((meta?.greens ?? []).map(line));
  const areas = $derived(meta?.areas ?? []);
  /** Outdoor places (a big world levels each to a height of its own: `pads`). */
  const spots = $derived(new Set(Object.keys(meta?.pads ?? {}).map(Number)));

  // ---- live: residents and the camera, twice a second while open ----
  let dots = $state.raw<{ x: number; z: number; mine: boolean }[]>([]);
  let camera = $state.raw<[number, number] | null>(null);
  $effect(() => {
    if (!game.townOpen) return;
    const poll = () => {
      dots = townMap.source.dots?.() ?? [];
      camera = townMap.source.camera?.() ?? null;
    };
    poll();
    const timer = setInterval(poll, 500);
    return () => clearInterval(timer);
  });

  // ---- pan (drag) and zoom (wheel) ----
  let svg = $state<SVGSVGElement>();
  let drag = $state.raw<{ x: number; y: number; cx: number; cz: number; moved: boolean } | null>(null);
  /** Whether the last pointer press was a drag (then it isn't a click). */
  let dragged = false;
  /** Where on the map the pointer is (m). */
  const at = (e: MouseEvent): [number, number] => {
    const r = svg!.getBoundingClientRect();
    return [x0 + ((e.clientX - r.left) / r.width) * viewW, z0 + ((e.clientY - r.top) / r.height) * viewD];
  };
  function onwheel(e: WheelEvent) {
    e.preventDefault();
    const [mx, mz] = at(e);
    const next = Math.max(1, Math.min(12, zoom * Math.exp(-e.deltaY * 0.0015)));
    // The point under the pointer stays under it.
    const k = zoom / next;
    mid = [mx + (cx - mx) * k, mz + (cz - mz) * k];
    zoom = next;
  }
  function onpointerdown(e: PointerEvent) {
    dragged = false;
    drag = { x: e.clientX, y: e.clientY, cx, cz, moved: false };
  }
  function onpointermove(e: PointerEvent) {
    if (!drag) return;
    const [dx, dy] = [e.clientX - drag.x, e.clientY - drag.y];
    if (!drag.moved && Math.hypot(dx, dy) < 4) return;
    if (!drag.moved) {
      svg?.setPointerCapture(e.pointerId);
      drag = { ...drag, moved: true };
    }
    const scale = viewW / (svg?.getBoundingClientRect().width || screenW);
    mid = [drag.cx - dx * scale, drag.cz - dy * scale];
  }
  function onpointerup() {
    dragged = !!drag?.moved;
    drag = null;
  }

  /** Where "Look here" takes the camera: the lot picked, or a place clicked on the map. */
  let target = $state<[number, number] | null>(null);
  function pickPlot(id: number) {
    if (dragged) return;
    picked = id;
    const p = plots[id];
    if (p) target = [p.x + p.w / 2, p.z + p.d / 2];
  }
  function pickPoint(e: MouseEvent) {
    if (!dragged) target = at(e);
  }
  function lookHere() {
    if (target) services.controls.lookAt(target[0], target[1]);
  }

  function status(s: (typeof game.sims)[number]): string {
    if (s.awayUntil !== null) return `At work until ${clock(s.awayUntil, settings.clock24h)}`;
    if (s.visiting !== null) return `Visiting ${game.plots[s.visiting]?.name ?? ''}`;
    if (s.plot === null) return 'Out and about';
    const hh = game.households[s.household];
    return s.plot === hh?.plot ? 'At home' : `At ${game.plots[s.plot]?.name}`;
  }
</script>

{#if game.townOpen}
  <!-- No backdrop: the world behind keeps its clicks and camera drags (`kit/popup.ts`). -->
  <div class="town glass" class:wide id="town-map" use:closeOutside={() => (game.townOpen = false)} role="dialog" aria-label="Town map">
    <header>
      <b>Town</b>
      {#if me && me.plot !== homePlot}
        <button class="btn small" onclick={() => services.controls.goHome()}><Icon name="icon.ui.home" size={14} /> {me.name}: go home</button>
      {/if}
      <button class="btn small" disabled={!target} title="Take the camera to the place picked on the map" onclick={lookHere}>Look here</button>
      {#if big}
        <button class="btn small" aria-pressed={wide} onclick={() => (wide = !wide)}>{wide ? 'Smaller' : 'Bigger'}</button>
      {/if}
      <button class="close" aria-label="Close" onclick={() => (game.townOpen = false)}><Icon name="icon.ui.close" size={14} /></button>
    </header>
    <div class="frame" bind:clientWidth={screenW} style:aspect-ratio="{fullW} / {fullD}">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <svg
        bind:this={svg}
        viewBox="{x0} {z0} {viewW} {viewD}"
        class="map"
        class:dragging={drag?.moved}
        {onwheel}
        {onpointerdown}
        {onpointermove}
        {onpointerup}
        onpointercancel={onpointerup}
      >
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <rect x={-MARGIN} y={-MARGIN} width={fullW} height={fullD} fill="#B9D7A2" onclick={pickPoint} />
        <g class="land">
          {#each zones as z, i (i)}
            <path class="zone {z.kind}" d={z.d} fill-rule="evenodd" />
          {/each}
          {#each greens as g, i (i)}
            <polygon class="green" points={g} />
          {/each}
          {#each roads as r, i (i)}
            {#if r.kind === 'trail'}
              <polyline class="trail" points={r.points} stroke-width={Math.max(r.width * 0.6, px(1.6))} stroke-dasharray="{px(5)} {px(3)}" />
            {:else}
              <polyline class="road {r.kind}" points={r.points} stroke-width={r.width} />
            {/if}
          {/each}
        </g>
        {#each plots as p (p.id)}
          {@const hh = game.households.find((h) => h.plot === p.id)}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <g class="plot" class:mine={p.id === homePlot} class:picked={p.id === picked} class:viewing={p.id === game.viewPlot} class:spot={spots.has(p.id)} onclick={() => pickPlot(p.id)}>
            <rect class="lot" x={p.x + 0.5} y={p.z + 0.5} width={p.w - 1} height={p.d - 1} rx="1" />
            {#if p.house}
              <rect class="house" x={p.house[0]} y={p.house[1]} width={p.house[2] - p.house[0]} height={p.house[3] - p.house[1]} rx="0.5" />
            {/if}
            <!-- Names where there's room for them on screen (outdoor places always). -->
            {#if p.w * ppm > 46 || spots.has(p.id)}
              {@const name = p.public ? p.name : hh ? `The ${hh.name}s` : 'Empty'}
              <text x={labelX(p.x + p.w / 2, name, 11)} y={labelZ(p.z + p.d / 2 + px(4), 11)} font-size={px(11)} text-anchor="middle">{name}</text>
              {#if homeCount(p.id) && p.w * ppm > 46}
                <text class="count" x={labelX(p.x + p.w / 2, `${homeCount(p.id)} here`, 9)} y={labelZ(p.z + p.d / 2 + px(17), 9)} font-size={px(9)} text-anchor="middle">{homeCount(p.id)} here</text>
              {/if}
            {/if}
          </g>
        {/each}
        {#each dots as d, i (i)}
          <circle class="dot" class:mine={d.mine} cx={d.x} cy={d.z} r={px(d.mine ? 4 : 3)} />
        {/each}
        {#each areas as a, i (i)}
          <text class="area" x={labelX(a.x + a.w / 2, a.name, 14)} y={labelZ(a.z + a.d / 2, 14)} font-size={px(14)} stroke-width={px(3)} text-anchor="middle">{a.name}</text>
        {/each}
        {#if camera}
          <circle class="camera" cx={camera[0]} cy={camera[1]} r={px(7)} stroke-width={px(2)} />
        {/if}
        {#if target}
          <circle class="target" cx={target[0]} cy={target[1]} r={px(5)} stroke-width={px(2)} />
        {/if}
      </svg>
      {#if zoom > 1}
        <button class="btn small reset" onclick={() => ((zoom = 1), (mid = null))}>Whole town</button>
      {/if}
    </div>

    {#if plot}
      <section class="details">
        <div class="title">
          <b>{household ? `The ${household.name}s` : plot.name}</b>
          <span class="muted">{plot.name}</span>
        </div>
        <ul>
          {#each residents as s (s.id)}
            {@const who = info(s.id)}
            <li>
              {#if who}<span class="face"><SimPreview appearance={who.appearance} gender={who.gender} id={who.id} size={30} /></span>{/if}
              <span class="who"><b>{s.name}</b><span class="muted">{status(s)}{s.job ? ` · ${s.job.title}` : ''}</span></span>
            </li>
          {/each}
        </ul>
        {#if me && plot.id !== me.plot && plot.id !== homePlot}
          <!-- (Greyed out, saying why, when the sim would refuse it.) -->
          <button class="btn primary" disabled={!!me.homebound} title={me.homebound} onclick={() => services.controls.visit(plot.id)}>
            Send {me.name} to visit
          </button>
          {#if me.homebound}<span class="muted why">{me.homebound}</span>{/if}
        {:else if me && plot.id === homePlot && me.plot !== homePlot}
          <button class="btn primary" onclick={() => services.controls.goHome()}>Send {me.name} home</button>
        {/if}
      </section>
    {:else}
      <p class="muted hint">Click a lot to see who lives there.</p>
    {/if}
  </div>
{/if}

<style>
  .town {
    position: absolute;
    top: var(--top);
    right: var(--edge);
    width: 460px;
    /* Short windows: it scrolls rather than running off the bottom. */
    max-height: calc(100% - var(--top) - var(--edge));
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    pointer-events: auto;
    animation: rise var(--slow) var(--ease);
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  header b {
    flex: 1;
    font-size: 16px;
  }
  .close {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    color: var(--text-muted);
  }
  .small {
    height: 30px;
    padding: 0 10px;
    font-size: 12px;
  }
  .town.wide {
    width: min(70vw, calc(100vw - 2 * var(--edge)));
  }
  .frame {
    position: relative;
    width: 100%;
    max-height: 70vh;
    overflow: hidden;
    border-radius: var(--radius-md);
  }
  .map {
    display: block;
    width: 100%;
    height: 100%;
    touch-action: none;
    cursor: grab;
  }
  .map.dragging {
    cursor: grabbing;
  }
  .reset {
    position: absolute;
    right: 8px;
    bottom: 8px;
  }
  .zone {
    opacity: 0.85;
  }
  .zone.water {
    fill: #8fbfe0;
  }
  .zone.woods {
    fill: #6f9a55;
  }
  .zone.hill {
    fill: #c3cc95;
    opacity: 0.6;
  }
  .zone.meadow {
    fill: #c8dd98;
  }
  .zone.fields {
    fill: #e4cf7f;
  }
  .land {
    pointer-events: none;
  }
  .green {
    fill: #9cc883;
  }
  .road {
    fill: none;
    stroke: #e9e4d8;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .road.country {
    stroke: #d6c39d;
  }
  .trail {
    fill: none;
    stroke: #8a6a3c;
    stroke-linecap: round;
  }
  .dot {
    fill: #f4f1ec;
    stroke: rgba(29, 34, 48, 0.6);
    stroke-width: 0.3;
    pointer-events: none;
  }
  .dot.mine {
    fill: var(--accent);
  }
  .area {
    font-weight: 750;
    fill: #1d2230;
    stroke: rgba(255, 255, 255, 0.85);
    paint-order: stroke;
  }
  .camera {
    fill: none;
    stroke: #1d2230;
    pointer-events: none;
  }
  .target {
    fill: none;
    stroke: var(--accent);
    pointer-events: none;
  }
  .spot .lot {
    fill: rgba(242, 179, 102, 0.55);
  }
  .plot {
    cursor: pointer;
  }
  .lot {
    fill: rgba(255, 255, 255, 0.2);
    stroke: rgba(255, 255, 255, 0.6);
    stroke-width: 0.3;
  }
  .plot:hover .lot,
  .picked .lot {
    fill: rgba(91, 124, 250, 0.2);
    stroke: var(--accent);
  }
  .viewing .lot {
    stroke-width: 0.8;
  }
  .house {
    fill: #f4f1ec;
    stroke: #c9c1b4;
    stroke-width: 0.3;
  }
  .mine .house {
    fill: var(--accent);
  }
  text {
    font-weight: 650;
    fill: rgba(29, 34, 48, 0.8);
    pointer-events: none;
  }
  .count {
    font-weight: 500;
  }
  .details {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .title {
    display: flex;
    flex-direction: column;
  }
  .muted {
    color: var(--text-muted);
    font-size: 12px;
  }
  .hint {
    margin: 0;
  }
  .why {
    margin-top: -4px;
    text-align: center;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .face {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    overflow: hidden;
    background: var(--surface-muted);
    display: grid;
    place-items: start center;
  }
  .who {
    display: flex;
    flex-direction: column;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
</style>
