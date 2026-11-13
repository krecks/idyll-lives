<script lang="ts">
  import Icon from './Icon.svelte';
  import { clock, money } from './format';
  import SimPreview from './kit/SimPreview.svelte';
  import { closeOutside } from './kit/popup';
  import { services } from './services';
  import { game } from './state.svelte';
  import { townMap } from './townMap.svelte';
  import { layoutLabels, mapScale, screenToMap, type MapLabel } from './mapView';
  import { lotTitle, wayText } from './place';
  import { settings } from '../settings/settings.svelte';
  import { homesForSale, type HomeForSale } from './town/places';

  /**
   * In-game town map: who lives where, who's home, and visiting. It pans (drag) and zooms (the
   * wheel or + and −, 1 to 12 times), its labels the same size on screen at any zoom; a big world's
   * land, country roads, trails (dashed), hamlets and outdoor places are on it too, and residents as
   * dots. "Look here" (or a double-click) takes the camera to the lot or place picked. The lots take
   * the keyboard too: Tab to one, Enter picks it, Enter again looks there.
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
  /** The lot picked, if it's for sale (nobody lives there): its price from the land and the house, and its size (`town/places.ts`). */
  const saleText = (h: HomeForSale) =>
    [
      'For sale',
      h.price && !game.creative ? `from ${money(h.price)}` : '',
      `${h.floor ? `${h.storeys > 1 ? `${h.storeys} floors, ` : ''}${Math.round(h.floor)} m² of house` : 'an empty lot'} on ${h.w} × ${h.d} m`,
    ]
      .filter(Boolean)
      .join(' · ');
  const sale = $derived(plot && !plot.public && !household ? homesForSale([plot], [], services.content.economy.homePrice)[0] : null);
  const homeCount = (plotId: number) => game.sims.filter((s) => s.plot === plotId).length;

  // ---- the view: zoom 1..12 round its middle (m); the map's width on screen (px) ----
  let zoom = $state(1);
  let mid = $state<[number, number] | null>(null);
  /** The frame's size (CSS px): the map is drawn to fit in it (`mapView.ts`). */
  let screenW = $state(432);
  let screenH = $state(300);
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
  const ppm = $derived(mapScale(screenW, screenH, { x: x0, z: z0, w: viewW, d: viewD }));
  const px = (n: number) => n / ppm;
  // ---- layers, worked out once per town ----
  const ZONE_ORDER = ['meadow', 'fields', 'hill', 'woods', 'orchard', 'water'];
  const line = (pts: [number, number][]) => pts.map(([x, z]) => `${x},${z}`).join(' ');
  const zones = $derived(
    (meta?.zones ?? []).map((z) => ({ kind: z.kind, d: `M${z.poly.map(([x, zz]) => `${x},${zz}`).join('L')}Z` })).sort((a, b) => ZONE_ORDER.indexOf(a.kind) - ZONE_ORDER.indexOf(b.kind)),
  );
  const roads = $derived((meta?.roads ?? []).map((r) => ({ kind: r.kind ?? 'street', width: r.width, points: line(r.points) })));
  const greens = $derived((meta?.greens ?? []).map(line));
  const areas = $derived(meta?.areas ?? []);
  /** Outdoor places (a big world levels each to a height of its own: `pads`). */
  const spots = $derived(new Set(Object.keys(meta?.pads ?? {}).map(Number)));

  // ---- names: laid out on screen as the town view's tags are (`mapView.ts` layoutLabels) ----
  /** A lot's name and the "2 here" under it, a place's name, a hamlet's (`a` and its index): apart, the hamlets' first. */
  const labels = $derived.by(() => {
    const list: MapLabel<number | string>[] = [];
    for (const p of plots) {
      // Names where there's room for them on screen (outdoor places always).
      const roomy = p.w * ppm > 46;
      if (!roomy && !spots.has(p.id)) continue;
      const wins = p.id === homePlot ? 0 : p.id === picked ? 1 : null;
      list.push({ key: p.id, x: p.x + p.w / 2, z: p.z + p.d / 2, text: lotTitle(p, game.households, 'Empty'), size: 11, sub: roomy ? 9 : undefined, rank: spots.has(p.id) ? 1 : 2, wins });
    }
    areas.forEach((a, i) => list.push({ key: `a${i}`, x: a.x + a.w / 2, z: a.z + a.d / 2, text: a.name, size: 14, rank: 0 }));
    // (Clear of the zoom buttons in the corner: two of 30 px, and "Whole town" zoomed in.)
    const zoomBox = { l: screenW - (zoom > 1 ? 160 : 74), t: screenH - 40, r: screenW, b: screenH };
    return layoutLabels(list, screenW, screenH, { x: x0, z: z0, w: viewW, d: viewD }, [zoomBox]);
  });

  // ---- live: residents twice a second while open; the camera every frame (it flies) ----
  let dots = $state.raw<{ x: number; z: number; mine: boolean }[]>([]);
  let camera = $state.raw<[number, number] | null>(null);
  $effect(() => {
    if (!game.townOpen) return;
    const poll = () => (dots = townMap.source.dots?.() ?? []);
    poll();
    const timer = setInterval(poll, 500);
    let frame = 0;
    const follow = () => {
      const c = townMap.source.camera?.() ?? null;
      // (A new value only when it moved a little: the map isn't drawn again for nothing.)
      if (!c || !camera || Math.abs(c[0] - camera[0]) > 0.25 || Math.abs(c[1] - camera[1]) > 0.25) camera = c;
      frame = requestAnimationFrame(follow);
    };
    follow();
    return () => {
      clearInterval(timer);
      cancelAnimationFrame(frame);
    };
  });

  // ---- pan (drag) and zoom (wheel) ----
  let svg = $state<SVGSVGElement>();
  let drag = $state.raw<{ x: number; y: number; cx: number; cz: number; moved: boolean } | null>(null);
  /** Whether the last pointer press was a drag (then it isn't a click). */
  let dragged = false;
  /** Where on the map the pointer is (m): the map fits its box, with bands to spare when the box isn't its shape. */
  const at = (e: MouseEvent): [number, number] => screenToMap(e.clientX, e.clientY, svg!.getBoundingClientRect(), { x: x0, z: z0, w: viewW, d: viewD });
  /** Zooms by `factor` round `[mx, mz]` (m), which stays where it is on screen. */
  function zoomBy(factor: number, mx = cx, mz = cz) {
    const next = Math.max(1, Math.min(12, zoom * factor));
    const k = zoom / next;
    mid = [mx + (cx - mx) * k, mz + (cz - mz) * k];
    zoom = next;
  }
  function onwheel(e: WheelEvent) {
    e.preventDefault();
    const [mx, mz] = at(e);
    zoomBy(Math.exp(-e.deltaY * 0.0015), mx, mz);
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
    const r = svg?.getBoundingClientRect();
    const scale = 1 / mapScale(r?.width || screenW, r?.height || screenH, { x: x0, z: z0, w: viewW, d: viewD });
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
  // A lot handed over to pick as the map opens (a home for sale's tag, `town/TownTags.svelte`): picked, shown.
  $effect(() => {
    const id = townMap.pick;
    if (!game.townOpen || id === null) return;
    townMap.pick = null;
    dragged = false;
    pickPlot(id);
    const p = plots[id];
    if (p) {
      zoom = Math.max(zoom, 3);
      mid = [p.x + p.w / 2, p.z + p.d / 2];
    }
  });
  function pickPoint(e: MouseEvent) {
    if (!dragged) target = at(e);
  }
  function lookHere() {
    if (target) services.controls.lookAt(target[0], target[1]);
  }
  /** Enter or Space on a lot picks it; again, looks there. (Its own: not the game's Space.) */
  function lotKey(e: KeyboardEvent, id: number) {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    e.stopPropagation();
    if (picked === id) lookHere();
    else pickPlot(id);
  }

  function status(s: (typeof game.sims)[number]): string {
    const way = wayText(s, game.households[s.household]?.plot ?? null, game.plots, game.households, (m) => clock(m, settings.clock24h));
    if (way) return way;
    if (s.awayUntil !== null) return `At work until ${clock(s.awayUntil, settings.clock24h)}`;
    if (s.visiting !== null) return `Visiting ${game.plots[s.visiting]?.name ?? ''}`;
    if (s.plot === null) return 'Out and about';
    const hh = game.households[s.household];
    return s.plot === hh?.plot ? 'At home' : `At ${game.plots[s.plot]?.name}`;
  }
</script>

{#if game.townOpen}
  <!-- No backdrop: the world behind keeps its clicks and camera drags (`kit/popup.ts`). -->
  <!-- Above the household strip (`--below`), its key and hint too: the "Bigger" map ran under the strip's cards. -->
  <div
    class="town glass"
    class:wide
    id="town-map"
    style:--below={game.stripHeight > 0 ? `calc(${game.stripHeight}px + var(--gap))` : '0px'}
    use:closeOutside={() => (game.townOpen = false)}
    role="dialog"
    aria-label="Town map"
  >
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
    <!-- The map's own shape, as wide as there's room for and no taller than 70 % of the window, nor than leaves room for the panel's
         header, key and hint above the strip (`--chrome`). -->
    <div
      class="frame"
      bind:clientWidth={screenW}
      bind:clientHeight={screenH}
      style:aspect-ratio="{fullW} / {fullD}"
      style:width="min(100%, calc(min(70vh, 100vh / var(--ui-scale) - var(--top) - var(--edge) - var(--below) - var(--chrome)) * {fullW} / {fullD}))"
    >
      <!-- (Pointer-only panning and zooming: the keyboard has the buttons and the lots.) -->
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
        <rect
          x={-MARGIN}
          y={-MARGIN}
          width={fullW}
          height={fullD}
          fill="#B9D7A2"
          onclick={pickPoint}
          ondblclick={(e) => {
            pickPoint(e);
            lookHere();
          }}
        />
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
          {@const name = lotTitle(p, game.households, 'Empty')}
          {@const at = labels.get(p.id)}
          <g
            class="plot"
            class:mine={p.id === homePlot}
            class:picked={p.id === picked}
            class:viewing={p.id === game.viewPlot}
            class:spot={spots.has(p.id)}
            role="button"
            tabindex="0"
            aria-label="{name}{p.public || name === 'Empty' ? '' : `, ${p.name}`}"
            aria-pressed={p.id === picked}
            onclick={() => pickPlot(p.id)}
            ondblclick={() => {
              pickPlot(p.id);
              lookHere();
            }}
            onkeydown={(e) => lotKey(e, p.id)}
          >
            <rect class="lot" x={p.x + 0.5} y={p.z + 0.5} width={p.w - 1} height={p.d - 1} rx="1" />
            {#if p.house}
              <rect class="house" x={p.house[0]} y={p.house[1]} width={p.house[2] - p.house[0]} height={p.house[3] - p.house[1]} rx="0.5" />
            {/if}
            <!-- Its name where there's room for it (`labels`). -->
            {#if at}
              <text x={at.x} y={at.y} font-size={px(11)} text-anchor="middle">{name}</text>
              {#if homeCount(p.id) && p.w * ppm > 46}
                <text class="count" x={at.x} y={at.subY} font-size={px(9)} text-anchor="middle">{homeCount(p.id)} here</text>
              {/if}
            {/if}
          </g>
        {/each}
        {#each dots as d, i (i)}
          <circle class="dot" class:mine={d.mine} cx={d.x} cy={d.z} r={px(d.mine ? 4 : 3)} />
        {/each}
        {#each areas as a, i (i)}
          {@const at = labels.get(`a${i}`)}
          {#if at}
            <text class="area" x={at.x} y={at.y} font-size={px(14)} stroke-width={px(3)} text-anchor="middle">{a.name}</text>
          {/if}
        {/each}
        {#if camera}
          <circle class="camera" cx={camera[0]} cy={camera[1]} r={px(7)} stroke-width={px(2)} />
        {/if}
        {#if target}
          <circle class="target" cx={target[0]} cy={target[1]} r={px(5)} stroke-width={px(2)} />
        {/if}
      </svg>
      <div class="zoom" role="group" aria-label="Zoom">
        {#if zoom > 1}
          <button class="btn small" onclick={() => ((zoom = 1), (mid = null))}>Whole town</button>
        {/if}
        <button class="btn small square" aria-label="Zoom in" title="Zoom in" disabled={zoom >= 12} onclick={() => zoomBy(1.6)}>+</button>
        <button class="btn small square" aria-label="Zoom out" title="Zoom out" disabled={zoom <= 1} onclick={() => zoomBy(1 / 1.6)}>−</button>
      </div>
    </div>
    <ul class="legend" aria-label="Map key">
      {#if spots.size}<li><span class="key place"></span>Outdoor place</li>{/if}
      <li><span class="key home"></span>Your home</li>
      <li><span class="key dot-mine"></span>Your household</li>
      <li><span class="key dot-other"></span>Residents</li>
      <li><span class="key ring-camera"></span>Camera</li>
      <li><span class="key ring-target"></span>Picked</li>
    </ul>

    {#if plot}
      <section class="details">
        <div class="title">
          <b>{household ? `The ${household.name}s` : plot.name}</b>
          <span class="muted">{plot.name}</span>
        </div>
        {#if sale}
          <p class="sale muted">{saleText(sale)}</p>
        {/if}
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
      <p class="muted hint">Click a lot to see who lives there; double-click anywhere to look there.</p>
    {/if}
  </div>
{/if}

<style>
  .town {
    position: absolute;
    top: var(--top);
    right: var(--edge);
    width: 460px;
    /* The panel's header, key and hint round the map (px): the map leaves room for them. */
    --chrome: 128px;
    /* Short windows: it scrolls rather than running off the bottom (or under the household strip). */
    max-height: calc(100% - var(--top) - var(--edge) - var(--below));
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
    flex: none;
    align-self: center;
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
  .zoom {
    position: absolute;
    right: 8px;
    bottom: 8px;
    display: flex;
    gap: 4px;
  }
  .square {
    width: 30px;
    padding: 0;
    justify-content: center;
    font-size: 16px;
  }
  .legend {
    flex-direction: row;
    flex-wrap: wrap;
    gap: 4px 12px;
    margin-top: -4px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .legend li {
    gap: 5px;
  }
  .key {
    flex: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }
  .key.place {
    border-radius: 2px;
    background: rgba(242, 179, 102, 0.85);
  }
  .key.home {
    border-radius: 2px;
    background: var(--accent);
  }
  .key.dot-mine {
    background: var(--accent);
    border: 1px solid rgba(29, 34, 48, 0.6);
  }
  .key.dot-other {
    background: #f4f1ec;
    border: 1px solid rgba(29, 34, 48, 0.6);
  }
  .key.ring-camera {
    border: 2px solid #1d2230;
  }
  .key.ring-target {
    border: 2px solid var(--accent);
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
  .zone.orchard {
    fill: #8fb25e;
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
    outline: none;
  }
  .plot:focus-visible .lot {
    stroke: var(--accent);
    stroke-width: 1.2;
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
  .hint,
  .sale {
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
