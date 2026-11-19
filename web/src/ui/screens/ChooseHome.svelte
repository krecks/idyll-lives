<script lang="ts">
  /**
   * New game, step 5: the home, chosen on the whole world. The camera opens on the whole land; the
   * chips under it glide to the main town or a village, and "Whole world" back out. The homes for
   * sale the household can afford are lit; two chips beside the legend add the rest ("All for sale"),
   * as a faint stone ring, or leave them out ("Within savings", where a Living game opens). In a
   * Creative game every home for sale is lit. A home under the pointer says its price and bedrooms,
   * and a click chooses it and glides in. The panel says who is moving
   * and what the chosen home is; under it the homes for sale as a list, for the keyboard, and the
   * main control on a narrow screen.
   *
   * A home is a house's lot or a flat (`homes.ts`): a building of flats, or the flats over a shop or a
   * café, is lit while one of its flats for sale is, and a click on it chooses its flat chosen already,
   * else the cheapest the household can afford; the card lists the building's other flats for sale,
   * and the list keeps a building's flats together under its name, each with its storey.
   *
   * A household of the town's (chosen in step 4, `app.play`) has its own home chosen already:
   * it may stay (the game takes it over as it is) or move to a home for sale, its old home then for
   * sale too. On Move in the rest of the homes for sale are filled (`newGame.ts` `moveInRequest`).
   */
  import { onDestroy, onMount } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import { flatsIdOf, houseTemplate, loadTemplates, slotDims, withGarden, type PlotSlot, type Templates } from '../../game/town';
  import { flatsTemplate } from '../../game/flats';
  import { GARDEN_SIZES, type GardenSize } from '../../game/housegen';
  import { lookAtAge, residentsOf, sleepersOf, type SimDraft } from '../../game/household';
  import { flatPriceParts, priceParts } from '../../game/price';
  import { householdFunds, jobTitle, savingsOf } from '../../game/wealth';
  import { settings } from '../../settings/settings.svelte';
  import { tierFor } from '../../game/people';
  import { furnishAhead } from '../../game/townJobs';
  import { householdsAhead, stepLabel } from '../newGame';
  import { flatName, homeKey, homeOffers, homeOnLot, homeRows, sameHome, storeyName, type HomeOffer, type HomeRef } from '../homes';
  import { app } from '../app.svelte';
  import { money } from '../format';
  import Icon from '../Icon.svelte';
  import Modal from '../kit/Modal.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import SimPreview from '../kit/SimPreview.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import SetupShell from './SetupShell.svelte';

  const { content, assets } = services;
  const economy = content.economy;
  let templates = $state.raw<Templates | null>(null);
  /** Changes when a lot's garden is made bigger or smaller (the lots along the street move). */
  const town = $derived(app.town!);
  /** The home of the town's household chosen in step 4 (null: a household made in Creation). */
  const from = app.residentHome;
  const household = app.mover!;
  const name = household.name;

  /** Every home of the town (houses and flats, `homes.ts`), once the templates are known. */
  const all = $derived(templates ? homeOffers(content, templates, town) : []);
  const lived = $derived(new Set(town.households.map(homeKey)));
  /** The homes for sale. */
  const vacant = $derived(all.filter((h) => !lived.has(h.key)));
  /** What can be chosen: the homes for sale, and the household's own home if it has one. */
  const choosable = $derived(from === null ? vacant : [...all.filter((h) => sameHome(h, from)), ...vacant]);
  const byKey = $derived(new Map(choosable.map((h) => [h.key, h])));
  /** The lots with a home to choose on them (a building's once for all its flats). */
  const choosableLots = $derived(new Set(choosable.map((h) => h.slot)));

  /**
   * The home chosen: a household of the town's opens on its own; one made in Creation on the home
   * chosen last time if it is still for sale, else (once the prices are known) the one it can afford nearest the town's middle.
   */
  let chosen = $state.raw<HomeRef | null>(from ?? app.home);
  const selected = $derived(chosen ? (byKey.get(homeKey(chosen)) ?? null) : null);
  /** The household stays in its own home (the game takes it over as it is). */
  const staying = $derived(sameHome(selected, from));
  $effect(() => {
    if (templates) app.home = selected ? refOf(selected) : null;
  });
  /** A home as the request names it (its lot, and its flat). */
  const refOf = (h: HomeRef): HomeRef => (h.flat === undefined ? { slot: h.slot } : { slot: h.slot, flat: h.flat });

  /** The chosen flat's building, and the flat as it would be furnished for sale (none for a house). */
  const building = $derived(templates && selected?.flat !== undefined ? flatsTemplate(templates, flatsIdOf(selected.lot)) : undefined);
  const flat = $derived(building && selected?.flat !== undefined ? building.units[selected.flat] : undefined);
  /** The flats of the chosen flat's building to choose from (its own, and the others for sale there), by storey. */
  const flatsHere = $derived(selected?.flat === undefined ? [] : homeRows(choosable.filter((h) => h.slot === selected!.slot), null));

  /** The house as it stands, or an empty lot to build on (not for a household staying in its own home, nor a flat). */
  let lot = $state<'house' | 'empty'>(app.emptyLot && from === null ? 'empty' : 'house');
  $effect(() => {
    app.emptyLot = lot === 'empty' && !staying && selected?.flat === undefined;
  });

  const garden = $derived<GardenSize | undefined>(selected?.flat === undefined ? selected?.lot.garden : undefined);
  /** The lot's size (m), or a flat's own. */
  const dims = $derived(templates && selected ? (flat ? { width: flat.rect.w, depth: flat.rect.d } : slotDims(templates, selected.lot)) : null);
  function setGarden(size: GardenSize) {
    if (templates && selected) app.town = withGarden(templates, town, selected.slot, size);
  }
  /** The garden sizes to choose from: one that doesn't fit beside the neighbours can't be picked. */
  const gardens = $derived(
    GARDEN_SIZES.map((value) => ({
      value,
      label: value[0].toUpperCase() + value.slice(1),
      disabled: !!templates && !!selected && value !== garden && withGarden(templates, town, selected.slot, value) === town,
    })),
  );

  const house = $derived(templates && selected && selected.flat === undefined ? houseTemplate(templates, selected.lot.template) : undefined);
  const residents = $derived(residentsOf(content, household));
  /**
   * Sleeping places in the home as it would be furnished for this household (empty lot, or their
   * own home: none asked), counted as the simulation counts them (sim-core `lifecycle::beds`): a slot
   * on anything with a sleep interaction, but a couple's bed (one where partners try for a baby) is
   * a place for one unless one of the household's couples sleeps in it.
   */
  const bedPlaces = $derived.by(() => {
    const sleep = content.sleepTags;
    if (!templates || !selected || staying || app.emptyLot || !sleep.length) return null;
    let objects: { def: string }[];
    if (selected.flat !== undefined) {
      // (A building's flats are furnished together, each for who lives there: this one for the household.)
      const furnished: (typeof residents | undefined)[] = [];
      furnished[selected.flat] = residents;
      const unit = flatsTemplate(templates, flatsIdOf(selected.lot), furnished)?.units[selected.flat];
      if (!unit) return null;
      objects = unit.layout.objects;
    } else {
      const fitted = houseTemplate(templates, selected.lot.template, residents);
      if (!fitted) return null;
      objects = [fitted, fitted.upper, fitted.basement].flatMap((floor) => floor?.objects ?? []);
    }
    let [places, couples] = [0, 0];
    for (const o of objects) {
      const def = content.object(o.def);
      if (!def || !content.offersTags(def, sleep)) continue;
      const slots = def.slots ?? 1;
      if (slots > 1 && def.interactions.some((i) => i.conceive)) {
        places += 1;
        couples += slots - 1;
      } else places += slots;
    }
    return places + Math.min(couples, residents.couples);
  });
  const sleepers = $derived(sleepersOf(residents));
  const shortage = $derived(bedPlaces !== null && sleepers > bedPlaces ? { beds: bedPlaces, sleepers } : null);
  /** Asking whether to move in anyway (`shortage` can't change while the dialog is open). */
  let confirming = $state(false);

  /** A Creative game: homes are free, money is no object. */
  const creative = $derived(app.mode === 'creative');
  /** What the household has saved: what it can pay for a home (Living). */
  const savings = savingsOf(content, household);
  /** More than the household has saved (Living): still there to look at, as a faint ring once "All for sale" is chosen. */
  const dear = (h: HomeOffer) => !creative && !sameHome(h, from) && h.price > savings;
  const dearCount = $derived(vacant.filter(dear).length);
  /** Which homes for sale the map lights (Living, with some too dear): those within their savings, or all of them, the rest faint. */
  let lit = $state<'within' | 'all'>('within');
  /** There is a choice to make: a Living game, the prices known, and some homes beyond their savings. */
  const litChoice = $derived(!creative && dearCount > 0);
  const showAll = $derived(!litChoice || lit === 'all');
  /** The lots lit: a home for sale on them within their savings (or any, all shown); and those whose every home for sale is too dear. */
  const litLots = $derived([...new Set(vacant.filter((h) => showAll || !dear(h)).map((h) => h.slot))]);
  const dearLots = $derived.by(() => {
    if (!litChoice || !showAll) return [];
    const within = new Set(vacant.filter((h) => !dear(h)).map((h) => h.slot));
    return [...new Set(vacant.filter((h) => !within.has(h.slot)).map((h) => h.slot))];
  });
  /** Bought as an empty lot: chosen so, or no house stands on it (as `assembleTown` sees it: money to build with besides). Never a flat. */
  const emptyLot = $derived(!staying && !!selected && selected.flat === undefined && (lot === 'empty' || (!!templates && !house)));
  /** The chosen home's price (an empty lot: the land alone), and what it's made of. */
  const parts = $derived(
    templates && selected && !staying
      ? selected.flat === undefined
        ? priceParts(content, templates, selected.lot, emptyLot)
        : flatPriceParts(content, templates, { template: flatsIdOf(selected.lot) }, selected.flat)
      : null,
  );
  /** The chosen home costs more than the household has saved: it can't move in (Living). */
  const tooDear = $derived(!creative && !!parts && parts.total > savings);
  /** What their own home cost them (a household of the town's). */
  const ownPrice = $derived(from === null ? undefined : all.find((h) => sameHome(h, from))?.price);
  /** What the household has to build with once it has bought its home (Living), or "free" (Creative). */
  const budget = $derived(creative ? 'Free: this is a Creative game' : money(staying ? householdFunds(content, savings, ownPrice ?? 0) : householdFunds(content, savings, parts?.total ?? 0, emptyLot)));

  /** A household's home in words: its lot's name, or its flat's. */
  const homeName = (h: HomeRef) => (h.flat === undefined ? (town.slots[h.slot]?.name ?? '') : flatName(town.slots[h.slot], h.flat));
  const neighbours = $derived(
    selected
      ? town.households
          .filter((h) => !sameHome(h, from) && !sameHome(h, selected))
          .map((h) => ({ ...h, key: homeKey(h), distance: Math.hypot(town.slots[h.slot].x - selected!.lot.x, town.slots[h.slot].z - selected!.lot.z) }))
          .sort((a, b) => a.distance - b.distance)
          .slice(0, 3)
      : [],
  );

  // ---- Where the camera is ---------------------------------------------------------------------

  const areas = $derived(town.open?.areas ?? []);
  const areaName = (s: PlotSlot) => areas[s.area ?? 0]?.name || town.name;
  /** Homes lit in each area (for sale; within their savings while only those are lit). */
  const forSaleIn = $derived(areas.map((_, k) => vacant.filter((h) => (h.lot.area ?? 0) === k && (showAll || !dear(h))).length));
  /** What the camera frames with no home in focus: the whole world, or one of its areas. */
  let shot = $state<'world' | { area: number }>('world');
  /** The camera is on the chosen home (false: on `shot`). */
  let close = $state(false);

  /** A chip: the camera glides to an area, or out to the whole world (the player's turns, zoom and pan let go). */
  function jump(to: 'world' | number) {
    close = false;
    shot = to === 'world' ? 'world' : { area: to };
    menuScene.reframe();
  }

  /** A lot clicked on the map or its tag: its home chosen (a building's: `homes.ts` `homeOnLot`), and the camera glides in. Empty ground: back to the shot. */
  function choose(plot: number | null) {
    const home = plot === null ? null : homeOnLot(choosable, plot, selected, from, dear);
    if (!home) {
      close = false;
      return;
    }
    pick(home);
  }

  /** A home chosen (from the list, the map or the card's flats), and the camera glides in. */
  function pick(home: HomeOffer) {
    chosen = refOf(home);
    if (sameHome(home, from)) lot = 'house';
    close = true;
  }

  // ---- Tags --------------------------------------------------------------------------------------

  let hover = $state<number | null>(null);
  let hoverOff = 0;
  /**
   * Tags show only for the home looked at, the one chosen and the household's own: crossing a little
   * empty ground on the way to a tag should not drop it at once. Hovering something else changes at once.
   */
  function setHover(plot: number | null): void {
    clearTimeout(hoverOff);
    if (plot !== null || hover === null) {
      hover = plot;
      return;
    }
    hoverOff = setTimeout(() => (hover = null), 160);
  }
  onDestroy(() => clearTimeout(hoverOff));

  /** A home's bedrooms in words; a house's lot with no house on it, "An empty lot". */
  function bedrooms(h: HomeOffer): string {
    if (h.flat === undefined && templates && !houseTemplate(templates, h.lot.template)) return 'An empty lot';
    return `${h.bedrooms} bedroom${h.bedrooms === 1 ? '' : 's'}`;
  }

  /** The chosen home's tag wins room first, then the one looked at. */
  const ORDER: readonly LotLabel['kind'][] = ['mine', 'sale', 'home', 'park'];

  const labels = $derived.by((): LotLabel[] => {
    const shown = [...new Set([selected?.slot ?? null, from?.slot ?? null, hover])].filter((p): p is number => p !== null && !!town.slots[p]);
    return shown.map((plot): LotLabel => {
      const s = town.slots[plot];
      if (plot === selected?.slot) {
        const sub = staying ? 'Their home' : emptyLot ? 'An empty lot' : selected.storey !== undefined ? `Your new home · ${storeyName(selected.storey)}` : 'Your new home';
        return { plot, title: `The ${name}s`, sub, kind: 'mine' };
      }
      if (plot === from?.slot) return { plot, title: `The ${name}s`, sub: 'Their home now', kind: 'home' };
      const sale = vacant.filter((h) => h.slot === plot);
      const flats = sale.filter((h) => h.flat !== undefined);
      if (flats.length) {
        // A building's flats for sale: how many, from what price, how many bedrooms.
        const cheapest = Math.min(...flats.map((h) => h.price));
        const beds = [...new Set(flats.map((h) => h.bedrooms))].sort((a, b) => a - b);
        const price = creative ? '' : ` · ${flats.length > 1 ? 'from ' : ''}${money(cheapest)}`;
        const title = `${flats.length === 1 ? 'Flat' : `${flats.length} flats`} for sale${price}`;
        const words = `${beds.length > 1 ? `${beds[0]}–${beds.at(-1)}` : beds[0]} bedroom${beds.at(-1) === 1 ? '' : 's'}`;
        return { plot, title, sub: `${words} · ${s.name}${flats.every(dear) ? ' · more than they have' : ''}`, kind: 'sale' };
      }
      const h = town.households.find((x) => x.slot === plot);
      if (h && s.kind === 'house') return { plot, title: `The ${h.household.name}s`, sub: s.name, kind: 'home' };
      if (sale.length) {
        const [home] = sale;
        return { plot, title: creative ? 'For sale' : `For sale · ${money(home.price)}`, sub: `${bedrooms(home)}${dear(home) ? ' · more than they have' : ''}`, kind: 'sale' };
      }
      return { plot, title: s.name, sub: (s.area ?? 0) > 0 ? areaName(s) : undefined, kind: 'park' };
    });
  });

  // ---- The list ----------------------------------------------------------------------------------

  /** The homes to choose from as the list has them (`homes.ts` `homeRows`: a building's flats together). */
  const rows = $derived(homeRows(choosable, from));
  /** A row's element id (a key's ":" left out, for the selector). */
  const rowId = (h: HomeOffer) => `home-${h.key.replace(':', '-')}`;
  let innerWidth = $state(window.innerWidth);
  /** Too narrow for the world beside the panel (as `SetupShell` lays it out): the list is the main control. */
  const narrow = $derived(innerWidth / settings.uiScale <= 760);
  let listOpen = $state(window.innerWidth / settings.uiScale <= 760);
  $effect(() => {
    if (narrow) listOpen = true;
  });
  /** The row the keyboard is on (its home lit on the map, its tag shown). */
  let active = $state(-1);
  let listEl = $state<HTMLElement>();

  function moveActive(to: number) {
    if (!rows.length) return;
    active = Math.max(0, Math.min(rows.length - 1, to));
    const row = rows[active];
    setHover(row.slot);
    listEl?.querySelector(`#${rowId(row)}`)?.scrollIntoView({ block: 'nearest' });
  }

  function listKey(e: KeyboardEvent) {
    const step: Record<string, number> = { ArrowDown: 1, ArrowUp: -1, PageDown: 8, PageUp: -8 };
    if (e.key in step) moveActive((active < 0 ? 0 : active) + step[e.key]);
    else if (e.key === 'Home') moveActive(0);
    else if (e.key === 'End') moveActive(rows.length - 1);
    else if ((e.key === 'Enter' || e.key === ' ') && active >= 0) pick(rows[active]);
    else return;
    e.preventDefault();
  }

  // ---- Who is moving -----------------------------------------------------------------------------

  /** "34 · Junior Developer": a resident's age and what they do. */
  function about(m: SimDraft): string {
    const retired = m.age >= content.retireAt;
    const work = !content.isAdult(m.age)
      ? (content.stageOf(m.age)?.label ?? '')
      : m.job
        ? `${retired ? 'Retired ' : ''}${jobTitle(content, m.job)}`
        : retired
          ? 'Retired'
          : 'Between jobs';
    return `${m.age} · ${work}`;
  }

  /** "Partner of Ben", "Child of Ann and Ben", "Sibling of Joe": a member's bonds within the household. */
  function bondsOf(uid: string): string {
    const nameOf = (u: string) => household.members.find((m) => m.uid === u)?.name;
    const names = (list: (string | undefined)[]) => list.filter(Boolean).join(' and ');
    const others = (preset: string, as: 'a' | 'b' | 'either') =>
      household.bonds
        .filter((b) => b.preset === preset && (as === 'either' ? b.a === uid || b.b === uid : b[as] === uid))
        .map((b) => nameOf(b.a === uid ? b.b : b.a));
    const lines = [
      ['Partner of', others('partners', 'either')],
      // (A parent bond runs from the child, `a`, to the parent, `b`.)
      ['Child of', others('parent', 'a')],
      ['Parent of', others('parent', 'b')],
      ['Sibling of', others('siblings', 'either')],
    ] as const;
    return lines
      .filter(([, list]) => list.some(Boolean))
      .map(([label, list]) => `${label} ${names([...list])}`)
      .join(' · ');
  }

  // ---- Moving in ---------------------------------------------------------------------------------

  function moveIn() {
    if (!selected || tooDear) return;
    if (shortage) {
      confirming = true;
      return;
    }
    start();
  }

  function start() {
    const request = selected && app.homeRequest(refOf(selected));
    if (!request) return;
    confirming = false;
    menuScene.highlight({});
    app.start(request);
  }

  function back() {
    app.screen = from === null ? 'create' : 'household';
  }

  onMount(async () => {
    templates = await loadTemplates();
    // The houses Move in fills furnished ahead in the town worker while the player chooses (left
    // running past this screen: Move in's town file waits behind it and finds them made).
    const seed = app.peopleSeed;
    if (seed !== undefined) {
      const ahead = householdsAhead(content, assets, templates, town, seed, household, tierFor(settings.graphics));
      furnishAhead(content, town, ahead).catch(() => {});
    }
    if (selected) return;
    const a = areas[0];
    const [cx, cz] = a ? [a.x + a.w / 2, a.z + a.d / 2] : [town.width / 2, town.depth / 2];
    const near = (h: HomeOffer) => ((h.lot.area ?? 0) === 0 ? 0 : 1e6) + Math.hypot(h.lot.x - cx, h.lot.z - cz);
    const first = vacant.filter((h) => !dear(h)).sort((p, q) => near(p) - near(q) || p.price - q.price)[0];
    chosen = first ? refOf(first) : null;
  });

  // The 3D town shows the chosen home as the game will (cleared when starting on an empty lot). A flat's building is
  // no house: it is shown with no home, in its own look as the town has it (`lotLooks`), lit and tagged as the home.
  $effect(() => {
    const shown = !selected || staying ? town : app.homeTown(selected);
    const home = selected ?? from;
    if (shown) void menuScene.show(shown, home && home.flat === undefined ? home.slot : null);
  });
  $effect(() => {
    menuScene.highlight({ hover, selected: selected?.slot ?? null, marked: litLots, dear: dearLots, outlines: false });
  });
  $effect(() => () => menuScene.highlight({}));

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

<svelte:window bind:innerWidth />

{#snippet head()}
  <h1 class="title">Where will the {name}s live?</h1>
{/snippet}

{#snippet panel()}
  {#if narrow}{@render homes()}{/if}
  <section class="group who" aria-labelledby="who-head">
    <h2 class="eyebrow" id="who-head">{from === null ? 'Moving in' : `Living at ${homeName(from)}`}</h2>
    <span class="family">The {name}s</span>
    <ul class="members">
      {#each household.members as m (m.uid)}
        {@const bonds = bondsOf(m.uid)}
        <li>
          <span class="face"><SimPreview appearance={lookAtAge(content, assets, m.appearance, m.age)} gender={m.gender} size={44} /></span>
          <span class="person">
            <span><b>{m.name}</b> <span class="about">{about(m)}</span></span>
            {#if bonds}<span class="bond">{bonds}</span>{/if}
            {#if m.traits.length || m.perks.length}
              <span class="tags">
                {#each m.traits as t (t)}<span class="trait">{content.trait(t)?.label ?? t}</span>{/each}
                {#each m.perks as p (p)}<span class="perk">{content.perk(p)?.label ?? p}</span>{/each}
              </span>
            {/if}
          </span>
        </li>
      {/each}
    </ul>
    {#if from !== null}
      <p class="hint">Settled residents: most already have a job and know a few of the neighbours. They can stay, or move to a home for sale.</p>
    {/if}
  </section>

  <section class="group home" aria-labelledby="home-head">
    {#if selected}
      <h2 class="eyebrow" id="home-head">{staying ? 'Their home' : 'The home chosen'}</h2>
      {#if house && !staying && selected.flat === undefined}
        <Segmented
          fill
          label="Start with"
          bind:value={lot}
          options={[
            { value: 'house', label: 'The house' },
            { value: 'empty', label: 'An empty lot' },
          ]}
        />
      {/if}
      <!-- A generated house's lot is cut to fit it: its garden can be made bigger or smaller. A farm keeps its size (its fields and farmyard come with it). -->
      {#if garden && selected.lot.lot !== 'farm' && !staying}
        <div class="lot-size">
          <span class="eyebrow sub">Garden</span>
          <Segmented label="Garden" bind:value={() => garden, setGarden} options={gardens} />
        </div>
      {/if}
      {#key `${selected.key}:${lot}:${selected.lot.template}`}
        <div class="card">
          <span class="eyebrow sub">{areaName(selected.lot)}</span>
          <h3>{selected.name}</h3>
          {#if flat && selected.storey !== undefined}
            <b class="house">A flat · {storeyName(selected.storey)}</b>
            <p>
              {selected.storey === 0 ? 'On the ground floor of' : building?.place ? 'Over' : 'Upstairs in'}
              {selected.lot.name}, in at the building's street door{selected.storey === 0 ? '' : ' and up its stairs'}. No garden of its own: inside, the walls and the furniture
              are theirs to change.
            </p>
          {:else if emptyLot}
            <b class="house">An empty lot</b>
            <p>The house is cleared away: build your own from the ground up.</p>
          {:else if house}
            <b class="house">{house.name}</b>
            <p>{house.description}</p>
          {/if}
          {#if flatsHere.length > 1}
            <!-- The building's other flats to choose from, each with its storey and price. -->
            <div class="flats" role="radiogroup" aria-label="Flats in {selected.lot.name}">
              {#each flatsHere as f (f.key)}
                {@const on = f.key === selected.key}
                <button class:on class:dear={dear(f)} role="radio" aria-checked={on} onclick={() => pick(f)}>
                  <b>Flat {(f.flat ?? 0) + 1}</b>
                  <small>{storeyName(f.storey ?? 0)} · {f.bedrooms} bed{f.bedrooms === 1 ? '' : 's'}{sameHome(f, from) ? ' · theirs' : creative ? '' : ` · ${money(f.price)}`}</small>
                </button>
              {/each}
            </div>
          {/if}
          <dl>
            {#if dims}
              <dt>{flat ? 'Flat' : 'Lot'}</dt>
              <dd>{dims.width} × {dims.depth} m{economy.rent ? ` · rent ${money(economy.rent.base + Math.round(economy.rent.perTile * dims.width * dims.depth))} a week` : ''}</dd>
            {/if}
            {#if flat}
              <dt>Bedrooms</dt>
              <dd>{flat.bedrooms}</dd>
              <dt>Building</dt>
              <dd>{building?.units.length} flat{building?.units.length === 1 ? '' : 's'} · {building?.storeys} storeys</dd>
            {:else if house && !emptyLot}
              <dt>Bedrooms</dt>
              <dd>{house.bedrooms}</dd>
            {/if}
            {#if staying && !creative && ownPrice !== undefined}
              <dt>Bought for</dt>
              <dd>{money(ownPrice)}</dd>
            {/if}
            {#if parts && !creative}
              <dt>Price</dt>
              <dd>
                {money(parts.total)}
                {#if parts.house || parts.furniture}
                  <small class="parts">land {money(parts.land)} · {flat ? 'flat' : 'house'} {money(parts.house)} · furniture {money(parts.furniture)}</small>
                {/if}
              </dd>
            {/if}
            {#if !creative}
              <dt>Saved</dt>
              <dd>{money(savings)}</dd>
            {/if}
            <dt>{staying ? 'To spend' : 'To build with'}</dt>
            <dd>{budget}</dd>
          </dl>
          {#if tooDear && parts}
            <p class="warn" role="status">
              Too expensive: the {name}s have {money(savings)} saved, {money(parts.total - savings)} short.{lot === 'empty' && !flat ? '' : flat ? ' Choose a home that costs less.' : ' Start on an empty lot, or choose a home that costs less.'}
            </p>
          {/if}
          {#if shortage}
            <p class="warn" role="status">
              Only {shortage.beds} bed place{shortage.beds === 1 ? '' : 's'} for {shortage.sleepers} sleepers. You can still move in: they'll share a bed or sleep on the floor until you add beds.
            </p>
          {/if}
          {#if from !== null && !staying}
            <p class="hint">Their home at {homeName(from)} goes up for sale, and another household may move in.</p>
          {/if}
          {#if neighbours.length}
            <span class="eyebrow sub">Closest neighbours</span>
            <ul class="near">
              {#each neighbours as n (n.key)}
                <li><b>The {n.household.name}s</b> · {homeName(n)}</li>
              {/each}
            </ul>
          {/if}
        </div>
      {/key}
    {:else if vacant.length}
      <h2 class="eyebrow" id="home-head">A home</h2>
      <p class="hint">
        Choose one of the {vacant.length} homes for sale on the map, houses and flats{creative ? '' : `: ${vacant.length - dearCount} within the ${money(savings)} they've saved`}. The chips under the map take you to the town and each village.
      </p>
    {:else}
      <p class="hint">Every home is taken. Go back and remove a neighbour household.</p>
    {/if}
    <p class="hint after">Once you move in, the other homes for sale find households too, but for a few.</p>
  </section>
  {#if !narrow}{@render homes()}{/if}
{/snippet}

<!-- The homes as a list: the keyboard's way round them, and the main control on a narrow screen (first in the panel there). -->
{#snippet homes()}
  <section class="group list">
    <button class="toggle" aria-expanded={listOpen} aria-controls="home-list" onclick={() => (listOpen = !listOpen)}>
      <span class="chevron" class:open={listOpen} aria-hidden="true"><Icon name="icon.ui.chevron" size={14} /></span>
      <span class="eyebrow">{from === null ? 'Homes for sale' : 'Homes to choose from'}</span>
      <span class="count">{rows.length}</span>
    </button>
    {#if listOpen}
      <ul
        id="home-list"
        class="homes"
        role="listbox"
        tabindex="0"
        aria-label="Homes to choose from"
        aria-activedescendant={active >= 0 && rows[active] ? rowId(rows[active]) : undefined}
        bind:this={listEl}
        onkeydown={listKey}
        onfocus={() => active < 0 && moveActive(Math.max(0, rows.findIndex((r) => r.key === selected?.key)))}
        onblur={() => setHover(null)}
      >
        {#each rows as r, i (r.key)}
          {#if r.flat !== undefined && (rows[i - 1]?.slot !== r.slot || sameHome(rows[i - 1], from))}
            <!-- A building's flats together, under its name. -->
            {@const n = rows.filter((x) => x.slot === r.slot && !sameHome(x, from)).length}
            <li class="building" role="presentation">{r.lot.name}{sameHome(r, from) ? '' : ` · ${n} flat${n === 1 ? '' : 's'} for sale`}</li>
          {/if}
          <!-- (The keys are the list's: an option of a listbox is not a stop of its own.) -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            id={rowId(r)}
            role="option"
            aria-selected={r.key === selected?.key}
            class:active={i === active}
            class:dear={dear(r)}
            class:own={sameHome(r, from)}
            class:unit={r.flat !== undefined}
            onclick={() => ((active = i), pick(r))}
            onmouseenter={() => setHover(r.slot)}
            onmouseleave={() => setHover(null)}
          >
            <span class="dot" aria-hidden="true"></span>
            <span class="addr">
              <b>{r.flat === undefined ? r.name : `Flat ${r.flat + 1} · ${storeyName(r.storey ?? 0)}`}</b>
              <small>{sameHome(r, from) ? 'Their home' : `${areaName(r.lot)} · ${bedrooms(r)}`}</small>
            </span>
            <span class="price">
              {#if !sameHome(r, from) && !creative}{money(r.price)}{/if}
              {#if dear(r)}<small>too dear</small>{/if}
            </span>
          </li>
        {/each}
      </ul>
      <p class="hint keys"><kbd>↑</kbd> <kbd>↓</kbd> look along the list · <kbd>Enter</kbd> choose</p>
    {/if}
  </section>
{/snippet}

{#snippet stage()}
  <TownStage
    {labels}
    order={ORDER}
    {hover}
    selected={selected?.slot ?? null}
    focus={close ? (selected?.slot ?? null) : null}
    {shot}
    selectable={(p) => choosableLots.has(p)}
    onhover={setHover}
    onselect={choose}
  />
  <div class="below" use:avoid>
    <div class="jump" role="toolbar" aria-label="Look at">
      <button class="chip" class:on={!close && shot === 'world'} aria-pressed={!close && shot === 'world'} onclick={() => jump('world')}><Icon name="icon.ui.town" size={14} /> Whole world</button>
      {#each areas as a, k (k)}
        {@const on = !close && typeof shot === 'object' && shot.area === k}
        <button class="chip" class:on aria-pressed={on} onclick={() => jump(k)}>
          {a.name || town.name}
          {#if forSaleIn[k]}<span class="n" title="{forSaleIn[k]} {showAll ? 'for sale' : 'within their savings'}">{forSaleIn[k]}</span>{/if}
        </button>
      {/each}
    </div>
    <div class="key">
      {#if litChoice}
        <div class="jump" role="radiogroup" aria-label="Homes lit on the map">
          <button class="chip" class:on={lit === 'within'} role="radio" aria-checked={lit === 'within'} onclick={() => (lit = 'within')}>
            Within savings <span class="n">{vacant.length - dearCount}</span>
          </button>
          <button class="chip" class:on={lit === 'all'} role="radio" aria-checked={lit === 'all'} onclick={() => (lit = 'all')}>
            All for sale <span class="n">{vacant.length}</span>
          </button>
        </div>
      {/if}
      <div class="legend menu-glass" aria-hidden="true">
        <span class="swatch sale"></span>
        {litChoice ? 'Within their savings' : 'For sale'}
        {#if litChoice && showAll}<span class="swatch dear"></span> More than they have{/if}
        <span class="sep"></span>
        Click a home to look closer · drag to turn · right-drag to move · scroll to zoom
      </div>
    </div>
  </div>
{/snippet}

{#snippet figures()}
  <span><b>{vacant.length}</b> for sale</span>
  {#if !creative && templates}<span><b>{vacant.length - dearCount}</b> within their savings</span>{/if}
  {#if areas.length > 1}<span><b>{areas.length - 1}</b> {areas.length === 2 ? 'village' : 'villages'}</span>{/if}
{/snippet}

{#snippet actions()}
  <button class="btn primary move-in" disabled={!selected || tooDear} title={!selected ? 'Choose a home first' : tooDear ? `The ${name}s can't afford this home` : undefined} onclick={moveIn}>
    {staying ? `Play the ${name}s →` : 'Move in →'}
  </button>
{/snippet}

<SetupShell label="The home" step={stepLabel(5)} back={{ label: 'Household', go: back }} {head} {panel} {stage} stats={figures} {actions} />

{#if confirming && shortage}
  <Modal title="Not enough beds" onclose={() => (confirming = false)} width={460}>
    <p>
      {shortage.sleepers} people are moving in, but this home has {shortage.beds} bed place{shortage.beds === 1 ? '' : 's'}. Extra residents will share a bed or sleep on the floor until you add beds.
    </p>
    <div class="actions">
      <button class="btn ghost" onclick={() => (confirming = false)}>Cancel</button>
      <button class="btn primary" onclick={start}>Move in anyway</button>
    </div>
  </Modal>
{/if}

<style>
  .title {
    margin: 2px 0 0;
    font-size: 22px;
    font-weight: 750;
    letter-spacing: -0.02em;
    line-height: 1.2;
    color: var(--text-strong);
  }
  .family {
    font-size: 18px;
    font-weight: 750;
    letter-spacing: -0.01em;
    color: var(--text-strong);
  }
  .members {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .members li {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .face {
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    overflow: hidden;
    background: var(--surface-muted);
    border: 2px solid var(--surface);
    box-shadow: var(--shadow-sm);
    display: grid;
    place-items: start center;
  }
  .person {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .about {
    font-size: 12px;
    color: var(--text-muted);
  }
  .bond {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
  }
  .trait,
  .perk {
    padding: 1px 7px;
    border-radius: var(--radius-pill);
    font-size: 11px;
    font-weight: 600;
    background: rgba(29, 34, 48, 0.06);
    color: #3b4254;
  }
  .perk {
    background: rgba(233, 162, 59, 0.16);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    animation: swap 360ms var(--ease) both;
  }
  h3 {
    margin: 0;
    font-size: 20px;
    letter-spacing: -0.02em;
    color: var(--text-strong);
  }
  .house {
    color: var(--accent);
    font-size: 14px;
  }
  .card p {
    margin: 0;
    font-size: 13px;
    color: #4a5163;
    line-height: 1.45;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 5px 14px;
    margin: 6px 0;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: rgba(29, 34, 48, 0.05);
    font-size: 13px;
    color: var(--text-strong);
  }
  dt {
    color: var(--text-muted);
  }
  dd {
    margin: 0;
    font-weight: 650;
  }
  .parts {
    display: block;
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-muted);
  }
  .card .warn {
    padding: 8px 10px;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--bad) 12%, transparent);
    color: var(--bad);
    font-weight: 600;
    font-size: 12.5px;
  }
  .near {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .lot-size {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .after {
    font-style: italic;
  }
  /* The list: a header that opens and closes it, then the homes, the keyboard's way round them. */
  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    margin: -4px 0;
    background: none;
    border: none;
    text-align: left;
    color: var(--text-strong);
  }
  /* (The icon points up: right while the list is closed, down while it is open.) */
  .chevron {
    display: grid;
    transform: rotate(90deg);
    transition: transform var(--fast) var(--ease);
  }
  .chevron.open {
    transform: rotate(180deg);
  }
  .count {
    margin-left: auto;
    padding: 1px 8px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.07);
    font-size: 12px;
    font-weight: 700;
  }
  .homes {
    list-style: none;
    margin: 0;
    padding: 3px;
    max-height: 320px;
    overflow: auto;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
    gap: 1px;
    border-radius: var(--radius-md);
    background: rgba(29, 34, 48, 0.04);
    outline: none;
  }
  .homes:focus-visible {
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .homes li {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 6px 9px;
    border-radius: calc(var(--radius-md) - 3px);
    font-size: 13px;
    color: var(--text-strong);
    cursor: pointer;
  }
  .homes li:hover,
  .homes li.active {
    background: rgba(255, 255, 255, 0.75);
  }
  .homes:focus-visible li.active {
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .homes li[aria-selected='true'] {
    background: var(--accent);
    color: #fff;
  }
  /* A building's name over its flats in the list; its flats indented under it. */
  .homes li.building {
    padding: 8px 9px 2px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--text-muted);
    cursor: default;
  }
  .homes li.building:hover {
    background: none;
  }
  .homes li.unit {
    padding-left: 18px;
  }
  /* The chosen flat's building's flats, to choose another. */
  .flats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 5px;
  }
  .flats button {
    text-align: left;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 5px 9px;
    border-radius: var(--radius-md);
    background: rgba(29, 34, 48, 0.05);
    border: 1px solid transparent;
    font-size: 12px;
    color: var(--text-strong);
    line-height: 1.25;
  }
  .flats button small {
    font-size: 11px;
    color: var(--text-muted);
  }
  .flats button.dear {
    opacity: 0.6;
  }
  .flats button.on {
    background: var(--accent);
    color: #fff;
  }
  .flats button.on small {
    color: rgba(255, 255, 255, 0.85);
  }
  .homes .dot {
    flex: none;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: rgb(255, 199, 97);
  }
  .homes li.dear .dot {
    background: none;
    box-shadow: inset 0 0 0 1.5px rgb(170, 164, 152);
  }
  .homes li.own .dot {
    background: var(--tag-home);
  }
  .addr {
    min-width: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }
  .addr b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .addr small,
  .price small {
    font-size: 11px;
    color: var(--text-muted);
  }
  .price {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
  }
  .homes li.dear .price {
    color: var(--text-muted);
  }
  .homes li[aria-selected='true'] small,
  .homes li[aria-selected='true'] .price {
    color: rgba(255, 255, 255, 0.85);
  }
  .keys kbd {
    font-size: 11px;
  }
  /* Under the world: the chips that glide to the town and each village, and what the colours say. */
  .below {
    position: absolute;
    left: 0;
    bottom: 0;
    z-index: 1;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    animation: rise 520ms 120ms var(--ease) both;
  }
  .jump {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: var(--radius-pill);
    background: var(--glass-menu);
    border: 1px solid var(--glass-menu-border);
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(12px) saturate(1.4);
    -webkit-backdrop-filter: blur(12px) saturate(1.4);
    font-size: 13px;
    font-weight: 650;
    color: var(--text-strong);
    transition: background var(--fast) var(--ease);
  }
  .chip:hover {
    background: var(--surface);
  }
  .chip.on {
    background: var(--accent);
    border-color: transparent;
    color: #fff;
  }
  .chip .n {
    min-width: 18px;
    padding: 0 5px;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--tag-sale) 30%, transparent);
    font-size: 11px;
    font-weight: 750;
    text-align: center;
  }
  .chip.on .n {
    background: rgba(255, 255, 255, 0.25);
  }
  .key {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border-radius: var(--radius-pill);
    font-size: 12px;
    font-weight: 550;
    color: #3b4254;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgb(255, 199, 97);
  }
  /* The faint stone ring the map draws round a home they can't afford. */
  .swatch.dear {
    margin-left: 6px;
    background: none;
    box-shadow: inset 0 0 0 1.5px rgb(170, 164, 152);
  }
  .sep {
    width: 1px;
    height: 14px;
    margin: 0 4px;
    background: var(--hairline);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 18px;
  }
  /* Too narrow for the world beside the panel: the hint about the mouse goes, and the list (first in
     the panel) is as long as the panel lets it be. */
  @container setup (max-width: 760px) {
    .legend {
      display: none;
    }
    .homes {
      max-height: none;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
  @keyframes swap {
    from {
      opacity: 0;
      transform: translateX(10px);
    }
  }
</style>
