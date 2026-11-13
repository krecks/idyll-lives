<script lang="ts">
  import { townView } from '../../render/shared/townView';
  import { settings } from '../../settings/settings.svelte';
  import { clock, money, WEEKDAYS } from '../format';
  import { services } from '../services';
  import { game } from '../state.svelte';
  import { townMap } from '../townMap.svelte';
  import { layoutTags, type Box, type TagSpot } from '../../game/tagLayout';
  import { homesForSale, nameLabels, townPlaces } from './places';
  import { homeTag, householdTags, nameTags, nearestFew, peopleTags, mostSales, placesInView, placeTags, saleTags, tagRank, yourPeople, type TagWorld, type TownTag } from './tags';
  import { MINE_SEEN, mineFade, TAGS_FROM, tagFade } from './fade';
  import { townHud } from './townHud.svelte';

  /**
   * The town tags: HTML labels over the canvas, each over the resident,
   * place, home or street it names, placed each frame through `townView.project`. They fade in with `townBlend`
   * from 0.2 (the household's own from the town view's start: `mineFade`) and show only the kinds the tag menu has on.
   * They are laid out as the menus' are (`game/tagLayout.ts`): the household's own first and always (moved clear of
   * each other when they stand together), then by kind (`tagRank`), each kept near its point or hidden, and all of
   * them inside the screen and clear of the HUD's panels. Of the places, only so many as the zoom has room for and
   * a hamlet's only while the camera looks into it (`placesInView`); a tag the pointer is on, or near the point of,
   * shows whatever it is. What they say is worked out from the views as they change; where they stand, every frame
   * (written straight to the elements: nothing reactive moves each frame).
   */
  const meta = $derived(townMap.world?.meta ?? null);
  const places = $derived(townPlaces(meta, game.plots));
  const placeOfPlot = $derived(new Map(places.flatMap((p, i) => (p.plot === undefined ? [] : [[p.plot, i] as [number, number]]))));
  const names = $derived(nameLabels(meta));
  const homePlot = $derived(game.households.find((h) => h.player)?.plot ?? null);
  const inTown = $derived(new Set(game.roster.map((s) => s.id)));
  const members = $derived(game.sims.filter((s) => inTown.has(s.id) && game.households[s.household]?.player));
  const memberIds = $derived(new Set(members.map((s) => s.id)));
  const stages = services.content.lifeStages;
  const time = (m: number) => clock(m, settings.clock24h);

  const world = $derived.by((): TagWorld | null => {
    if (!townHud.near || game.mode !== 'live') return null;
    return {
      sims: game.sims.filter((s) => inTown.has(s.id)),
      roster: game.roster,
      households: game.households,
      plots: game.plots,
      relationships: game.relationships,
      places,
      placeOfPlot,
      minute: game.minute,
      weekday: game.weekday,
      time,
      days: WEEKDAYS,
      pupil: (stage) => !!stages.find((s) => s.id === stage)?.school,
      areas: meta?.areas ?? [],
    };
  });
  const people = $derived(world ? yourPeople(memberIds, world) : []);
  /** The household's residents away at no place the town has: the Today card lists them. */
  const household = $derived(world ? householdTags(members, world) : { tags: [], away: [] });
  $effect(() => {
    townHud.away = household.away.map((a) => ({ id: a.sim.id, name: a.sim.name, text: a.text }));
  });

  const tags = $derived.by((): TownTag[] => {
    if (!world) return [];
    const f = townHud.filters;
    const out: TownTag[] = [];
    if (f.household) {
      out.push(...household.tags);
      const home = homeTag(homePlot === null ? null : game.plots[homePlot], members, homePlot);
      if (home) out.push(home);
    }
    // (Up to the blend the rest come in at, the household's own only.)
    if (!townHud.on) return out;
    if (f.people) out.push(...peopleTags(people, world, homePlot));
    if (f.places) {
      const yours = new Map<number, string>([...members.map((s): [number, string] => [s.id, s.name]), ...people.map((p): [number, string] => [p.sim, game.sims.find((s) => s.id === p.sim)?.name ?? ''])]);
      out.push(...placeTags(world, yours));
    }
    if (f.sale) out.push(...saleTags(homesForSale(game.plots, game.households, services.content.economy.homePrice), game.plots, money));
    if (f.names) out.push(...nameTags(names));
    return out;
  });
  // ---- each frame: where they stand, which are kept ----
  const pins = new Map<string, HTMLElement>();
  /** Each tag's size (HUD px), measured when its words change. */
  const sizes = new Map<string, { text: string; w: number; h: number }>();
  /** The tags shown last frame and their boxes (HUD px): they keep their places first, and the pointer on one keeps it. */
  let shown = new Map<string, Box>();
  /** The tags moved off their point last frame (their stem is hidden). */
  const edges = new Set<string>();
  let layer = $state<HTMLDivElement>();
  const spot = { x: 0, y: 0, z: 0 };
  /** The pointer over the town or the tags (HUD px), or null. */
  let pointer: { x: number; y: number } | null = null;

  function pin(el: HTMLElement, key: string) {
    pins.set(key, el);
    return {
      update(next: string) {
        pins.delete(key);
        key = next;
        pins.set(key, el);
      },
      destroy() {
        if (pins.get(key) === el) pins.delete(key);
      },
    };
  }

  /** Where a tag's point is (town metres), or null when its resident isn't anywhere to be seen. */
  function pointOf(t: TownTag): boolean {
    const src = townMap.source;
    if ('sim' in t.anchor) {
      if (!src.spot?.(t.anchor.sim, spot)) return false;
    } else {
      spot.x = t.anchor.x;
      spot.z = t.anchor.z;
      spot.y = src.ground?.(spot.x, spot.z) ?? 0;
    }
    spot.y += t.lift;
    return true;
  }

  /** A pill's stem (px) under its box. */
  const STEM = 10;
  /** The pointer this near (px) a tag's point brings its tag up, though there was no room for it or it isn't up at this zoom. */
  const HOVER_REACH = 24;
  /** Margin (px) kept between the tags and the screen's edge, and round the HUD's panels. */
  const EDGE = 8;
  const PANEL = 4;

  /** A tag up for this frame: its point on screen (HUD px) and its place in the layout. */
  interface Candidate {
    key: string;
    t: TownTag;
    /** Up at this zoom (else only with the pointer on it). */
    up: boolean;
    spot: TagSpot;
  }

  const byKey = $derived(new Map(tags.map((t) => [t.key, t])));
  function frame() {
    const blend = game.mode === 'live' ? townView.blend : 0;
    const on = blend >= TAGS_FROM - 0.01 && !!townView.project;
    if (on !== townHud.on) townHud.on = on;
    const fade = Math.round(tagFade(blend) * 20) / 20;
    if (fade !== townHud.fade) townHud.fade = fade;
    const mine = Math.round(mineFade(blend) * 20) / 20;
    if (mine !== townHud.mine) townHud.mine = mine;
    // (The household's own are there once they can be seen: a tag too faint to see is not there to be clicked.)
    const near = mine >= MINE_SEEN && !!townView.project;
    if (near !== townHud.near) townHud.near = near;
    if (!near || !layer) {
      // (The layer goes with them: a new one starts with nothing shown.)
      shown.clear();
      edges.clear();
      return;
    }
    const scale = settings.uiScale || 1;
    const project = townView.project!;
    const chosen = townHud.chosen;
    const chosenKey = chosen?.kind === 'person' ? `p:${chosen.sim}` : chosen?.kind === 'place' ? `pl:${chosen.place}` : null;
    const look = townView.target;
    const radius = townView.radius;
    // (Of the homes for sale, the few nearest where the camera looks; of the places, as many as the zoom has room for.)
    const sales: { key: string; far: number }[] = [];
    for (const t of tags) if (t.kind === 'sale' && 'x' in t.anchor) sales.push({ key: t.key, far: Math.hypot(t.anchor.x - look.x, t.anchor.z - look.z) });
    const saleUp = nearestFew(sales, mostSales(radius));
    const placesUp = placesInView(tags, meta?.areas ?? [], look, radius);
    const area: Box = { l: EDGE, t: EDGE, r: innerWidth / scale - EDGE, b: innerHeight / scale - EDGE };
    const all: Candidate[] = [];
    for (const [key, el] of pins) {
      const t = byKey.get(key);
      if (!t) continue;
      if (t.radius && (radius < t.radius[0] || radius > t.radius[1])) continue;
      const up = t.kind === 'sale' ? saleUp.has(key) : t.kind === 'places' ? placesUp.has(key) : true;
      // (One not up at this zoom is still looked at for the pointer, if it has something to do.)
      if (!up && (!pointer || !t.action) && key !== chosenKey) continue;
      if (!pointOf(t)) continue;
      const p = project(spot.x, spot.y, spot.z);
      if (!p) continue;
      const x = p[0] / scale;
      const y = p[1] / scale;
      // (Its point on the screen, or no tag: one pulled in from off it would point at nothing.)
      if (x < area.l || x > area.r || y < area.t || y > area.b) continue;
      const text = `${t.title}|${t.detail}`;
      let size = sizes.get(key);
      if (!size || size.text !== text) {
        const tag = el.firstElementChild as HTMLElement | null;
        size = { text, w: tag?.offsetWidth ?? 0, h: tag?.offsetHeight ?? 0 };
        sizes.set(key, size);
      }
      // (A name is centred on its point, a pill stands on its stem over it.)
      const name = t.kind === 'names';
      const h = name ? size.h : size.h + STEM;
      all.push({ key, t, up, spot: { sx: x, sy: name ? y + size.h / 2 : y, w: size.w, h, rank: tagRank(t), capped: false, wins: null, held: shown.has(key), x: 0, y: 0, crowded: false } });
    }
    // The tag the pointer is on (as shown last frame), else the one whose point it is nearest.
    let hovered: string | null = null;
    if (pointer) {
      const { x, y } = pointer;
      let best = HOVER_REACH;
      for (const c of all) {
        if (!c.t.action || c.t.kind === 'household') continue;
        const b = shown.get(c.key);
        if (b && x >= b.l && x <= b.r && y >= b.t && y <= b.b) {
          hovered = c.key;
          break;
        }
        const d = Math.hypot(x - c.spot.sx, y - c.spot.sy);
        if (d < best) {
          best = d;
          hovered = c.key;
        }
      }
    }
    const spots: TagSpot[] = [];
    const placed: Candidate[] = [];
    for (const c of all) {
      // The clicked tag, the household's own and the one under the pointer win: shown however crowded it is.
      c.spot.wins = c.key === chosenKey ? 0 : c.t.kind === 'household' ? 1 : c.key === hovered ? 2 : null;
      if (!c.up && c.spot.wins === null) continue;
      spots.push(c.spot);
      placed.push(c);
    }
    const blocks = panels(scale).map((o) => ({ l: o.x - PANEL, t: o.y - PANEL, r: o.x + o.w + PANEL, b: o.y + o.h + PANEL }));
    layoutTags(spots, area, blocks);
    const next = new Map<string, Box>();
    for (const { key, t, spot: s } of placed) {
      if (s.crowded) continue;
      const el = pins.get(key)!;
      next.set(key, { l: s.x - s.w / 2, t: s.y - s.h, r: s.x + s.w / 2, b: s.y });
      const name = t.kind === 'names';
      el.style.transform = `translate(${s.x.toFixed(1)}px, ${(name ? s.y - s.h / 2 : s.y).toFixed(1)}px)`;
      el.style.visibility = 'visible';
      const edge = !name && (Math.abs(s.x - s.sx) > 2 || Math.abs(s.y - s.sy) > 2);
      if (edge !== edges.has(key)) {
        if (edge) edges.add(key);
        else edges.delete(key);
        el.classList.toggle('edge', edge);
      }
    }
    for (const key of shown.keys()) if (!next.has(key)) pins.get(key)?.style.setProperty('visibility', '');
    shown = next;
  }

  /** The HUD's panels (its glass, and the place pill), which no tag covers: looked up twice a second (HUD px). */
  let panelBoxes: { x: number; y: number; w: number; h: number }[] = [];
  let panelsAt = 0;
  function panels(scale: number) {
    const now = performance.now();
    if (now - panelsAt < 500 || !layer) return panelBoxes;
    panelsAt = now;
    const hud = layer.parentElement;
    panelBoxes = [...(hud?.querySelectorAll<HTMLElement>('.glass, :scope > .place') ?? [])].flatMap((el) => {
      const r = el.getBoundingClientRect();
      return r.width && r.height ? [{ x: r.left / scale, y: r.top / scale, w: r.width / scale, h: r.height / scale }] : [];
    });
    return panelBoxes;
  }
  /** The pointer over the town (the canvas) or a tag: where it is (HUD px); anywhere else (a panel, off the window), none. */
  function track(e: PointerEvent) {
    const over = e.target instanceof Element && (e.target.matches('canvas.scene') || !!layer?.contains(e.target));
    const scale = settings.uiScale || 1;
    pointer = over && e.pointerType !== 'touch' ? { x: e.clientX / scale, y: e.clientY / scale } : null;
  }
  $effect(() => {
    let raf = 0;
    const loop = () => {
      raf = requestAnimationFrame(loop);
      frame();
    };
    raf = requestAnimationFrame(loop);
    const leave = () => (pointer = null);
    addEventListener('pointermove', track, { passive: true });
    document.documentElement.addEventListener('pointerleave', leave);
    return () => {
      cancelAnimationFrame(raf);
      removeEventListener('pointermove', track);
      document.documentElement.removeEventListener('pointerleave', leave);
      pointer = null;
      townHud.on = false;
      townHud.fade = 0;
      townHud.near = false;
      townHud.mine = 0;
    };
  });

  function passWheel(e: WheelEvent) {
    document.querySelector('canvas.scene')?.dispatchEvent(new WheelEvent('wheel', e));
  }

  /** A click on a tag: the camera glides there; the household's resident is selected, your people and places open a card, a home for sale opens on the town map. */
  function choose(t: TownTag) {
    const a = t.action;
    if (!a) return;
    const at = pointOf(t) ? { x: spot.x, z: spot.z } : null;
    if (a.kind === 'select') {
      if (at) townView.goTo?.({ x: at.x, z: at.z, select: a.sim });
      return;
    }
    if (a.kind === 'home') {
      if (at) townView.goTo?.({ x: at.x, z: at.z, radius: 24 });
      return;
    }
    if (at) townView.goTo?.({ x: at.x, z: at.z, radius: 60 });
    if (a.kind === 'person') townHud.chosen = { kind: 'person', sim: a.sim, plot: a.plot, title: t.title, detail: t.detail };
    else if (a.kind === 'place') townHud.chosen = { kind: 'place', place: a.place, plot: places[a.place]?.plot ?? null, title: t.title, detail: t.detail };
    else {
      townHud.chosen = null;
      townMap.pick = a.plot;
      game.townOpen = true;
      game.journalOpen = false;
    }
  }
</script>

{#if townHud.near && game.mode === 'live'}
  <!-- (The wheel over a tag zooms the camera as it would over the town: passed on to the canvas.) -->
  <div class="tags" bind:this={layer} style:--fade={townHud.fade} style:--mine={townHud.mine} aria-label="Town" onwheel={passWheel}>
    {#each tags as t (t.key)}
      <div class="pin" class:name={t.kind === 'names'} class:own={t.kind === 'household'} use:pin={t.key}>
        {#if t.action}
          <button class="tag {t.tone}" onclick={() => choose(t)} title={t.detail ? `${t.title} · ${t.detail}` : t.title}>
            <span class="dot" aria-hidden="true"></span>
            <b>{t.title}</b>
            {#if t.detail}<span class="detail">{t.detail}</span>{/if}
          </button>
        {:else}
          <span class="tag {t.tone}">{t.title}</span>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .tags {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }
  /* (Hidden until a frame places it and keeps it: `frame`.) */
  .pin {
    position: absolute;
    left: 0;
    top: 0;
    visibility: hidden;
    will-change: transform;
    opacity: var(--fade);
  }
  /* (The household's own: in from the town view's start, and over the others where they had to stand on them.) */
  .pin.own {
    opacity: var(--mine);
    z-index: 1;
  }
  /* (Moved off its point to find room: no stem pointing at something it doesn't stand over.) */
  .pin:global(.edge) .tag::after {
    display: none;
  }
  /* The menus' lot tags' look (`kit/TownStage.svelte`): a pill on a stem, a coloured dot. One line: plain, short. */
  .tag {
    position: absolute;
    left: 0;
    bottom: 10px;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px 4px 8px;
    border-radius: var(--radius-pill);
    background: var(--tag-bg);
    color: var(--text-strong);
    box-shadow: var(--tag-shadow);
    white-space: nowrap;
    font-size: 12px;
    line-height: 1.2;
    pointer-events: auto;
    transition:
      transform var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
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
  button.tag:hover {
    transform: translateX(-50%) translateY(-2px);
    box-shadow: var(--tag-shadow-hover);
    background: #fff;
  }
  .tag b {
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .detail {
    font-size: 11px;
    font-weight: 550;
    color: var(--text-muted);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
    background: var(--tag-park);
  }
  .mine {
    background: var(--accent);
    color: #fff;
  }
  .mine::after {
    background: var(--accent);
  }
  button.mine:hover {
    background: var(--accent);
  }
  .mine .detail {
    color: rgba(255, 255, 255, 0.86);
  }
  .mine .dot {
    background: #fff;
  }
  .person .dot {
    background: var(--tag-home);
  }
  .closed .dot {
    background: var(--text-muted);
  }
  .closed b {
    color: var(--text-muted);
  }
  .sale .dot {
    background: var(--tag-sale);
  }
  /* Street and area names: words on the land, no pill, no stem; not clickable. */
  .name .tag {
    bottom: auto;
    top: 0;
    transform: translate(-50%, -50%);
    padding: 0;
    background: none;
    box-shadow: none;
    pointer-events: none;
    color: var(--text-strong);
    text-shadow:
      0 0 3px rgba(255, 255, 255, 0.95),
      0 0 6px rgba(255, 255, 255, 0.8);
  }
  .name .tag::after {
    display: none;
  }
  .street {
    font-size: 11px;
    font-weight: 600;
    font-style: italic;
  }
  .area {
    font-size: 16px;
    font-weight: 750;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
</style>
