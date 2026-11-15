<script lang="ts">
  import { onMount } from 'svelte';
  import type { WorldOptions } from '../../core/protocol';
  import { WORLD_FLAG } from '../../debug/benchFlags';
  import { menuScene } from '../../game/menuScene';
  import { loadTemplates, NEIGHBOUR_CAP, type NeighbourhoodDraft, type Templates } from '../../game/town';
  import { keepHouses, Superseded, townJob } from '../../game/townJobs';
  import { defaultsOf, LEGACY_KINDS, placeOf, specOf, WORLD_KINDS, WORLDS, worldName, type WorldKind, type WorldStep } from '../../game/world';
  import { OPTION_FIELDS, sameOptions, worldLine } from '../../game/worldOptions';
  import type { TownBuildStep } from '../../render/types';
  import { app } from '../app.svelte';
  import { toast } from '../state.svelte';
  import { lifespanHint } from '../format';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';

  const { content } = services;
  let templates = $state.raw<Templates | null>(null);
  let world = $state.raw<NeighbourhoodDraft | null>(app.world);
  // The place and options: the world's own, back from step 2; else the countryside's (or the URL's,
  // `?world=`, for the timing tools; a kind from before places is its place at its size).
  const asked = app.world ? null : WORLD_FLAG;
  const place = placeOf(app.world?.world?.kind ?? asked?.kind ?? 'country');
  let kind = $state<WorldKind>(place);
  app.worldOptions ??= app.world?.world?.options ?? { ...defaultsOf(place), ...(asked && Object.hasOwn(LEGACY_KINDS, asked.kind) ? { town: LEGACY_KINDS[asked.kind].town } : {}), ...asked?.options };
  /** The options the next world is made with (kept in `app` while going to step 2 and back). */
  const options = $derived(app.worldOptions ?? defaultsOf(kind));
  let name = $state(app.world?.name ?? '');
  /** The name was typed or rolled here (kept when the world is made again). */
  let named = $state(false);
  let hover = $state<number | null>(null);
  let focus = $state<number | null>(null);

  const homes = $derived(world ? world.slots.filter((s) => s.kind === 'house').length : 0);
  const farms = $derived(world ? world.slots.filter((s) => s.lot === 'farm').length : 0);
  const places = $derived(world ? world.slots.filter((s) => s.kind !== 'house').length : 0);
  const roadCount = $derived(world ? world.roads.length : 0);
  const villages = $derived(world?.open ? world.open.areas.length - 1 : 0);
  /** The world in a line: homes, its town's size, land, how many residents it holds. */
  const sizeHint = $derived(world ? worldLine({ homes, villages, width: world.width, depth: world.depth, town: world.world?.options?.town, cap: NEIGHBOUR_CAP }) : '');

  // Public places and farms have their names on them; plain houses are left untagged (the town reads better).
  // A big world's areas each pick their own shops, so one name can come up in two of them: a repeat
  // says which hamlet it is in (the town's own needs no saying).
  const labels = $derived.by((): LotLabel[] => {
    const w = world;
    if (!w) return [];
    const shown = w.slots.filter((s) => s.kind !== 'house' || s.lot === 'farm');
    const times = new Map<string, number>();
    for (const s of shown) times.set(s.name, (times.get(s.name) ?? 0) + 1);
    return shown.map((s) => {
      const area = s.area ?? 0;
      return {
        plot: s.index,
        title: s.name,
        sub: area > 0 && (times.get(s.name) ?? 0) > 1 ? w.open?.areas[area]?.name : undefined,
        kind: s.kind === 'house' ? 'sale' : 'park',
      };
    });
  });

  /**
   * Making a world, as the progress panel lists it: laid out in the town worker (`WorldStep`, in
   * turn), then built in 3D (`TownBuildStep`, side by side: they tick off in any order).
   */
  const STEPS: readonly { key: WorldStep | TownBuildStep; label: string }[] = [
    { key: 'land', label: 'Shaping the land' },
    { key: 'roads', label: 'Laying the roads' },
    { key: 'homes', label: 'Placing the homes' },
    { key: 'houses', label: 'Building the houses' },
    { key: 'hills', label: 'Raising the hills' },
    { key: 'woods', label: 'Planting the woods' },
    { key: 'town', label: 'Building the town' },
    { key: 'grass', label: 'Sowing the grass' },
  ];

  /** The world being made (laid out in the town worker, then built); a newer one replaces it. */
  let job: AbortController | null = null;
  /** A world is being made: the progress panel shows (over the last one, if there is one). */
  let making = $state(false);
  /** It is the same world with other options (`setOption`), not another one. */
  let changing = $state(false);
  /** The world last asked for: its seed and the name it was asked with (none: its seed's own). */
  let wanted: { seed: number; name?: string } | null = null;
  /** An option changed: the world is made again once they settle (`setOption`); meanwhile it can't be chosen. */
  let settling: ReturnType<typeof setTimeout> | undefined;
  let waiting = $state(false);
  /** Its steps done so far. */
  let done = $state.raw<ReadonlySet<string>>(new Set());
  /** The first step not done yet (the one under way while the world is laid out). */
  const current = $derived(STEPS.find((s) => !done.has(s.key))?.key ?? null);
  const progress = $derived(done.size / STEPS.length);

  /** A new seed: a different place every time. */
  const roll = () => (Math.random() * 2 ** 31) >>> 0;
  /** The name as typed or rolled here, if any (a new world keeps it). */
  const typedName = () => (named && name.trim() ? name.trim() : undefined);

  /**
   * A world of the chosen place with the chosen options from `seed`, named `asName` (else its
   * seed's name), laid out and built; the one being made meanwhile is dropped.
   */
  async function make(seed: number, asName?: string, again = false) {
    clearTimeout(settling);
    waiting = false;
    if (!templates) return;
    job?.abort();
    const control = (job = new AbortController());
    wanted = { seed, name: asName };
    making = true;
    changing = again;
    done = new Set();
    const tick = (step: WorldStep | TownBuildStep) => {
      if (job === control) done = new Set([...done, step]);
    };
    // (Nothing in view yet: the first world is built in longer slices.)
    const first = !world;
    try {
      const made = await townJob(content, { kind: 'world', world: kind, seed, name: asName, options: { ...options } }, control.signal, tick);
      if (job !== control) return;
      // Its houses and preview come with it: the menu draws it without making them again.
      keepHouses(made);
      menuScene.prepare(made.draft, made.preview);
      world = made.draft;
      name = asName ?? world.name;
      focus = null;
      // (The panel stays until it is built and on screen.)
      await menuScene.show(world, null, tick, first);
    } catch (err) {
      if (err instanceof Superseded) return;
      console.error('[world] making the world failed:', err);
      toast('That world could not be made. Try another.');
    } finally {
      if (job === control) {
        job = null;
        making = false;
      }
    }
  }
  $effect(() => () => {
    clearTimeout(settling);
    job?.abort();
  });

  /** Another world of the chosen place (the options as they are). */
  function another() {
    void make(roll(), typedName());
  }

  /** Another place: a new world there, with the place's own options. */
  function choose(k: WorldKind) {
    if (k === kind && world) return;
    kind = k;
    app.worldOptions = defaultsOf(k);
    void make(roll(), typedName());
  }

  /**
   * An option changed: the same world (its seed and name) made again with it, once the options
   * have stood for 300 ms (a click or an arrow key each changes one; only the last is made). A world
   * still being made is made again so: its seed, and the name it was asked with.
   */
  function setOption(key: keyof WorldOptions, value: string) {
    if (options[key] === value) return;
    app.worldOptions = { ...options, [key]: value };
    clearTimeout(settling);
    waiting = true;
    settling = setTimeout(() => {
      const seed = making ? wanted?.seed : world?.seed;
      if (seed === undefined) return void (waiting = false);
      void make(seed, making ? wanted?.name : name.trim() || undefined, true);
    }, 300);
  }

  function rollName() {
    name = worldName(content, specOf(kind, options).names, Math.random);
    named = true;
  }

  function next() {
    if (!world) return;
    const chosen = { ...world, name: name.trim() || world.name };
    // The same world as before (back and forth): its neighbours stay.
    const same = app.town && app.town.seed === chosen.seed && app.town.world?.kind === chosen.world?.kind && sameOptions(app.town.world?.options, chosen.world?.options);
    app.town = same && app.town ? { ...app.town, name: chosen.name } : null;
    app.world = chosen;
    app.screen = 'neighbourhood';
  }

  onMount(async () => {
    templates = await loadTemplates();
    // Back from step 2: the world made before (still on screen). Else a new one is made now.
    if (world) void menuScene.show(world);
    else {
      // (Not the town the main menu glided away from, if it is still there.)
      void menuScene.hide();
      await make(roll());
    }
  });

  /** Leaving for the main menu: its picture is taken of the world in view (`App.svelte`). */
  function toMenu() {
    clearTimeout(settling);
    job?.abort();
    app.toMenu();
  }

  $effect(() => {
    menuScene.highlight({ hover, selected: focus, marked: [], outlines: true });
  });
  $effect(() => () => menuScene.highlight({}));

  /** Keeps lot tags clear of this element. */
  const avoid = (el: HTMLElement) => ({ destroy: menuScene.avoid(el) });
</script>

{#snippet glyph(k: WorldKind)}
  <svg viewBox="0 0 48 32" width="48" height="32" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round">
    {#if k === 'country'}
      <!-- Rolling hills, a church and a tree on them, a lane winding up. -->
      <path d="M2 18c6-5 13-6 20-3M22 15c7-4 16-4 24 0M9 15v-4h4v4M9 11l2-5 2 5M38 13V11M35 8a3 3 0 1 1 6 0 3 3 0 1 1-6 0M2 27h44M22 27c-1-2 0-4 3-5s3-3 1-4" />
    {:else if k === 'river'}
      <!-- A humpbacked bridge over the water. -->
      <path d="M2 17c10-7 34-7 44 0M2 21c10-5 34-5 44 0M15 27c0-4 4-6 9-6s9 2 9 6M2 27c2 0 3-1 5-1s3 1 5 1M36 27c2 0 3-1 5-1s3 1 5 1" />
    {:else if k === 'heath'}
      <!-- A tor on the moor's brow, tufts of heather. -->
      <path d="M2 21c5-4 11-6 17-7M28 14c7 1 13 3 18 7M19 14v-3h3V8h5v3h2v3zM6 27l-1-4M8 27v-5M10 27l1-4M22 27l-1-3M24 27v-4M26 27l1-3M38 27l-1-4M40 27v-5M42 27l1-4" />
    {:else if k === 'island'}
      <!-- An island with a lighthouse, the sea all round. -->
      <path d="M8 23c2-6 8-9 16-9s14 3 16 9M2 27c3-2 5-2 8 0s5 2 8 0 5-2 8 0 5 2 8 0 5-2 8 0M2 23h3M43 23h3M21 14l1-8h4l1 8M21 6h6M24 6V3" />
    {:else if k === 'farm'}
      <path d="M6 27V15l9-7 9 7v12M11 27v-6h8v6M30 27V9a4 4 0 0 1 8 0v18M2 27h44M38 18h6M41 15v12" />
    {:else if k === 'seaside'}
      <path d="M14 17V9l6-5 6 5v8M4 21c3-2 5-2 8 0s5 2 8 0 5-2 8 0 5 2 8 0 5-2 8 0M4 27c3-2 5-2 8 0s5 2 8 0 5-2 8 0 5 2 8 0 5-2 8 0" />
    {:else if k === 'mountain'}
      <path d="M2 27 16 7l8 11 6-7 16 16zM12 13l4 3 3-3M27 15l3 2 3-2" />
    {:else if k === 'lakeside'}
      <path d="M8 23c0-4 7-6 16-6s16 2 16 6-7 5-16 5-16-1-16-5zM14 23h8M28 25h4M38 15V7M38 7l-4 6h8z" />
    {:else}
      <path d="M10 27V21M10 21l-6 0 6-9 6 9zM10 14l-4 0 4-7 4 7zM24 27V19M24 19h-7l7-12 7 12zM38 27V21M38 21h-6l6-9 6 9zM38 14h-4l4-7 4 7zM2 27h44" />
    {/if}
  </svg>
{/snippet}

<div class="screen scaled">
  <div class="haze" aria-hidden="true"></div>
  <header use:avoid>
    <button class="btn ghost back" onclick={toMenu}><Icon name="icon.ui.back" size={18} /> Menu</button>
    <div class="title">
      <span class="eyebrow">Step 1 of 4 · A new world</span>
      <div class="name-field">
        <input class="input town-name" aria-label="Town name" placeholder={world ? "" : "Naming the town…"} bind:value={name} oninput={() => (named = true)} maxlength="24" />
        <button class="btn ghost roll" aria-label="Another name" title="Another name" onclick={rollName}><Icon name="icon.ui.dice" size={18} /></button>
      </div>
      {#if world?.world}
        {#key world.seed}<p class="lede">{world.world.tagline}</p>{/key}
      {/if}
    </div>
    <div class="actions">
      <button class="btn" disabled={!templates} onclick={another}><Icon name="icon.ui.dice" size={18} /> Another world</button>
      <button class="btn primary" disabled={!world || making || waiting} onclick={next}>Choose a household →</button>
    </div>
  </header>

  <main>
    <section class="view">
      {#if making}
        <div class="making menu-glass" role="status" aria-live="polite" aria-label="Making the world">
          <span class="eyebrow">{changing ? 'Changing the world' : world ? 'Another world' : 'A new world'}</span>
          <div class="bar"><div class="fill" style:transform="scaleX({progress})"></div></div>
          <ol>
            {#each STEPS as s (s.key)}
              <li class:done={done.has(s.key)} class:now={s.key === current}>
                <span class="mark" aria-hidden="true">{#if done.has(s.key)}✓{:else if s.key === current}<span class="spinner"></span>{/if}</span>
                {s.label}
              </li>
            {/each}
          </ol>
        </div>
      {/if}
      {#if world}
        <TownStage {labels} {hover} selected={focus} {focus} onhover={(p) => (hover = p)} onselect={(p) => (focus = p === focus ? null : p)} />
      {:else}
        <div></div>
      {/if}
      {#if world}
        <div class="controls menu-glass" use:avoid>
          <div class="row yours" role="group" aria-label="Your world">
            <div class="control summary">
              <span class="eyebrow">Your world</span>
              <span class="hint">{sizeHint}</span>
            </div>
            {#each OPTION_FIELDS as f (f.key)}
              <div class="control option">
                <span class="eyebrow">{f.label}</span>
                <Segmented label={f.label} value={options[f.key]} options={f.choices} onchange={(v) => setOption(f.key, v)} />
              </div>
            {/each}
          </div>
          <div class="row">
            <div class="control">
              <span class="eyebrow">Game</span>
              <Segmented
                label="Game"
                bind:value={app.mode}
                options={[
                  { value: 'living', label: 'Living' },
                  { value: 'creative', label: 'Creative' },
                ]}
              />
              <span class="hint">{app.mode === 'creative' ? 'Building is free. No rent or bills at home.' : 'Your residents earn the money to build with.'}</span>
            </div>
            <div class="control">
              <span class="eyebrow">Lifespan</span>
              <Segmented
                label="Lifespan"
                bind:value={app.lifespan}
                options={[
                  { value: 'off', label: 'Off' },
                  { value: 'short', label: 'Short' },
                  { value: 'normal', label: 'Normal' },
                  { value: 'long', label: 'Long' },
                ]}
              />
              <span class="hint">{lifespanHint(app.lifespan)}</span>
            </div>
            <div class="stats" aria-label="World summary">
              <span><b>{roadCount}</b> {roadCount === 1 ? 'road' : 'roads'}</span>
              <span><b>{homes}</b> homes</span>
              {#if farms}<span><b>{farms}</b> {farms === 1 ? 'farm' : 'farms'}</span>{/if}
              <span><b>{places}</b> public {places === 1 ? 'place' : 'places'}</span>
            </div>
          </div>
        </div>
      {/if}
    </section>

    <aside class="kinds menu-glass" use:avoid>
      <header class="list-head">
        <span class="eyebrow">Where it takes place</span>
      </header>
      <div class="list" role="radiogroup" aria-label="Kind of world">
        {#each WORLD_KINDS as k (k)}
          <button class="kind" role="radio" aria-checked={kind === k} class:on={kind === k} disabled={!templates} onclick={() => choose(k)}>
            <span class="glyph">{@render glyph(k)}</span>
            <span class="info">
              <b>{WORLDS[k].label}</b>
              <span>{WORLDS[k].blurb}</span>
            </span>
          </button>
        {/each}
      </div>
    </aside>
  </main>
</div>

<style>
  .screen {
    position: relative;
    /* (Viewport units count in the UI scale.) */
    height: calc(100vh / var(--ui-scale));
    container: setup / inline-size;
    display: grid;
    /* As wide as the screen, whatever the header's buttons would like. */
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto 1fr;
    padding: 20px 28px 24px;
    gap: 14px;
  }
  /* Light haze behind the header so its text reads on any part of the scene. */
  .haze {
    position: absolute;
    inset: 0 0 auto 0;
    height: 150px;
    background: var(--scene-haze);
    pointer-events: none;
  }
  header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 20px;
  }
  .back {
    color: var(--text-strong);
  }
  .title {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .title .eyebrow {
    color: #4a5163;
  }
  /* The name is a field you can type in, not a heading: a surface and a hairline of its own at rest,
     a solid field under the pointer, the accent ring while it has the caret. The dice (roll another
     name) sits in the field's end. */
  .name-field {
    display: flex;
    align-items: center;
    max-width: 440px;
    padding: 0 4px 0 12px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--surface) 62%, transparent);
    transition:
      background var(--fast) var(--ease),
      border-color var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
  .name-field:hover {
    background: var(--surface);
  }
  .name-field:focus-within {
    background: var(--surface);
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .name-field .town-name {
    flex: 1;
    min-width: 0;
    height: 46px;
    padding: 0;
    font-size: 26px;
    font-weight: 750;
    letter-spacing: -0.025em;
    color: var(--text-strong);
    background: transparent;
    border: none;
    box-shadow: none;
  }
  .name-field .town-name:focus {
    border: none;
    box-shadow: none;
  }
  .roll {
    flex: none;
    width: 34px;
    height: 34px;
    padding: 0;
    color: #4a5163;
  }
  .lede {
    margin: 0;
    font-size: 14px;
    font-weight: 550;
    font-style: italic;
    color: #3b4254;
    animation: rise 420ms var(--ease) both;
  }
  .actions {
    display: flex;
    gap: 10px;
  }
  main {
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr minmax(360px, 420px);
    /* One row as tall as the screen allows: a long side panel scrolls rather than stretching the town view. */
    grid-template-rows: minmax(0, 1fr);
    gap: 24px;
  }
  /* Too narrow for the town beside the panel (a small window or a large UI scale): the town on
     top, the panel under it. */
  @container setup (max-width: 760px) {
    .screen > header {
      flex-wrap: wrap;
      row-gap: 10px;
    }
    /* The title keeps a readable width; the buttons take the next line when they must. */
    .screen > header .title {
      flex: 1 1 300px;
    }
    main {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(0, 1.2fr) minmax(0, 1fr);
      gap: 14px;
    }
    .kinds {
      align-self: stretch;
    }
  }
  .view {
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-rows: 1fr auto;
    gap: 14px;
    /* The stream pill floats at the window's bottom edge, centred (StreamIndicator): the controls
       bar and the stats stay clear above it rather than under it. */
    padding-bottom: calc(70px / var(--ui-scale));
  }
  /* The world's making, step by step: in the middle of the stage (over the last world, if any, which stays in view meanwhile). */
  .making {
    position: absolute;
    inset: 0;
    margin: auto;
    width: min(320px, 100%);
    height: max-content;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px 24px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-strong);
    pointer-events: none;
    animation: rise 260ms var(--ease) both;
  }
  .making .bar {
    height: 6px;
    border-radius: 3px;
    overflow: hidden;
    background: color-mix(in srgb, var(--text-strong) 12%, transparent);
  }
  .making .fill {
    height: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform 300ms var(--ease);
  }
  .making ol {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 6px;
  }
  .making li {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
    transition: color var(--fast) var(--ease);
  }
  .making li.now,
  .making li.done {
    color: var(--text-strong);
  }
  .making .mark {
    flex: none;
    width: 16px;
    display: grid;
    place-items: center;
    color: var(--accent);
  }
  .making .mark .spinner {
    width: 12px;
    height: 12px;
  }
  /* Two rows: the world's options (with the world in a line), then the game's; each wraps when narrow. */
  .controls {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 20px;
    justify-self: start;
    max-width: 100%;
    animation: rise 520ms var(--ease) both;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 28px;
  }
  .row.yours {
    align-items: flex-start;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--hairline);
  }
  .control {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 200px;
  }
  .control.option {
    min-width: 0;
  }
  .summary .hint {
    max-width: 230px;
  }
  .hint {
    font-size: 12px;
    color: var(--text-muted);
    max-width: 210px;
  }
  .controls .eyebrow,
  .list-head .eyebrow {
    color: #4a5163;
  }
  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    color: #3b4254;
    font-size: 13px;
    font-weight: 550;
  }
  .stats span {
    padding: 5px 10px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.06);
  }
  .stats b {
    color: var(--text-strong);
    font-weight: 750;
  }
  .kinds {
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: 16px 12px 12px 16px;
    align-self: start;
    max-height: 100%;
    animation: rise 520ms 80ms var(--ease) both;
  }
  .list-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 0 6px 10px 2px;
    border-bottom: 1px solid var(--hairline);
    margin-bottom: 8px;
  }
  .list {
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-right: 4px;
  }
  .kind {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 12px;
    text-align: left;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    color: var(--text-strong);
    transition:
      background var(--fast) var(--ease),
      border-color var(--fast) var(--ease);
  }
  .kind:hover {
    background: rgba(255, 255, 255, 0.75);
  }
  .kind.on {
    background: #fff;
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .glyph {
    flex: none;
    display: grid;
    place-items: center;
    width: 60px;
    height: 44px;
    border-radius: var(--radius-md);
    background: var(--surface-muted);
    color: #4a5163;
  }
  .kind.on .glyph {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .info span {
    color: var(--text-muted);
    font-size: 12.5px;
    font-weight: 550;
  }
  /* Narrow (as above): the town gets what height there is. The stream pill floats over the places'
     list instead (its last place scrolls clear of it), the world's line takes the panel's width, and
     the stats, which that line already sums up, are left out. */
  @container setup (max-width: 760px) {
    .view {
      padding-bottom: 0;
    }
    .list {
      padding-bottom: calc(56px / var(--ui-scale));
    }
    .summary {
      flex-basis: 100%;
      flex-flow: row wrap;
      align-items: baseline;
      gap: 4px 12px;
    }
    .summary .hint {
      max-width: none;
    }
    .stats {
      display: none;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
