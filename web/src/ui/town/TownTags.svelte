<script lang="ts">
  import { townView } from '../../render/shared/townView';
  import { settings } from '../../settings/settings.svelte';
  import { clock, money, WEEKDAYS } from '../format';
  import { services } from '../services';
  import { game } from '../state.svelte';
  import { townMap } from '../townMap.svelte';
  import { keepApart, type TagBox } from './overlap';
  import { homesForSale, nameLabels, townPlaces } from './places';
  import { householdTags, nameTags, peopleTags, placeTags, saleTags, yourPeople, type TagWorld, type TownTag } from './tags';
  import { TAGS_FROM, tagFade } from './fade';
  import { townHud } from './townHud.svelte';

  /**
   * The town tags (docs/design/living-town.md, Town tags): HTML labels over the canvas, each over the resident,
   * place, home or street it names, placed each frame through `townView.project`. They fade in with `townBlend`
   * from 0.2, show only the kinds the tag menu has on, and are kept from overlapping, the more important one
   * staying (`overlap.ts`). What they say is worked out from the views as they change; where they stand, every frame
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
    if (!townHud.on || game.mode !== 'live') return null;
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
    if (f.household) out.push(...household.tags);
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
  let shown = new Set<string>();
  let layer = $state<HTMLDivElement>();
  const spot = { x: 0, y: 0, z: 0 };

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

  const byKey = $derived(new Map(tags.map((t) => [t.key, t])));
  function frame() {
    const blend = game.mode === 'live' ? townView.blend : 0;
    const on = blend >= TAGS_FROM - 0.01 && !!townView.project;
    if (on !== townHud.on) townHud.on = on;
    const fade = Math.round(tagFade(blend) * 20) / 20;
    if (fade !== townHud.fade) townHud.fade = fade;
    if (!on || !layer) return;
    const scale = settings.uiScale || 1;
    const project = townView.project!;
    const boxes: TagBox[] = [];
    const where = new Map<string, [number, number]>();
    const chosen = townHud.chosen;
    const chosenKey = chosen?.kind === 'person' ? `p:${chosen.sim}` : chosen?.kind === 'place' ? `pl:${chosen.place}` : null;
    for (const [key, el] of pins) {
      const t = byKey.get(key);
      if (!t) continue;
      if (t.radius && (townView.radius < t.radius[0] || townView.radius > t.radius[1])) continue;
      if (!pointOf(t)) continue;
      const p = project(spot.x, spot.y, spot.z);
      if (!p) continue;
      const text = `${t.title}|${t.detail}`;
      let size = sizes.get(key);
      if (!size || size.text !== text) {
        const tag = el.firstElementChild as HTMLElement | null;
        size = { text, w: tag?.offsetWidth ?? 0, h: tag?.offsetHeight ?? 0 };
        sizes.set(key, size);
      }
      const x = p[0] / scale;
      const y = p[1] / scale;
      where.set(key, [x, y]);
      const name = t.kind === 'names';
      const climb = t.kind === 'household' ? 3 : t.kind === 'people' ? 2 : 0;
      boxes.push({ key, kind: t.kind, x: x - size.w / 2, y: name ? y - size.h / 2 : y - 10 - size.h, w: size.w, h: size.h, depth: p[2], pinned: key === chosenKey, climb });
    }
    const kept = keepApart(boxes, { width: innerWidth / scale, height: innerHeight / scale, shown, avoid: panels(scale) });
    shown = new Set(kept.keys());
    for (const [key, el] of pins) {
      const at = where.get(key);
      const dy = kept.get(key);
      if (at && dy !== undefined) {
        el.style.transform = `translate(${at[0].toFixed(1)}px, ${(at[1] + dy).toFixed(1)}px)`;
        el.style.visibility = 'visible';
      } else el.style.visibility = '';
    }
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
  $effect(() => {
    let raf = 0;
    const loop = () => {
      raf = requestAnimationFrame(loop);
      frame();
    };
    raf = requestAnimationFrame(loop);
    return () => {
      cancelAnimationFrame(raf);
      townHud.on = false;
      townHud.fade = 0;
    };
  });

  /** A click on a tag: the camera glides there; the household's resident is selected, your people and places open a card, a home for sale opens on the town map. */
  function choose(t: TownTag) {
    const a = t.action;
    if (!a) return;
    const at = pointOf(t) ? { x: spot.x, z: spot.z } : null;
    if (a.kind === 'select') {
      if (at) townView.goTo?.({ x: at.x, z: at.z, select: a.sim });
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

{#if townHud.on && game.mode === 'live'}
  <div class="tags" bind:this={layer} style:opacity={townHud.fade} aria-label="Town">
    {#each tags as t (t.key)}
      <div class="pin" class:name={t.kind === 'names'} use:pin={t.key}>
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
