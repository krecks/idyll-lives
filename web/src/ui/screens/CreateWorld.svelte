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

  /**
   * Arrow keys in the place grid: across by one (wrapping round), up and down by a row, as many as
   * the grid shows in a row (three beside the town, more under it when narrow). A radio group: a move
   * chooses the place, as a click does.
   */
  function onPlaceKey(e: KeyboardEvent) {
    const grid = e.currentTarget as HTMLElement;
    const row = getComputedStyle(grid).gridTemplateColumns.split(' ').length;
    const step = ({ ArrowRight: 1, ArrowLeft: -1, ArrowDown: row, ArrowUp: -row } as Record<string, number>)[e.key];
    if (!step) return;
    e.preventDefault();
    if (!templates) return;
    const n = WORLD_KINDS.length;
    const at = WORLD_KINDS.indexOf(kind);
    const next = Math.abs(step) === 1 ? (at + step + n) % n : at + step;
    if (next < 0 || next >= n) return;
    choose(WORLD_KINDS[next]);
    (grid.children[next] as HTMLElement | undefined)?.focus();
  }

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
  <div class="layout">
    <aside class="side menu-glass" use:avoid aria-label="A new world">
      <div class="head" use:avoid>
        <button class="btn ghost back" onclick={toMenu}><Icon name="icon.ui.back" size={16} /> Menu</button>
        <span class="eyebrow">Step 1 of 4 · A new world</span>
        <div class="name-field">
          <input class="input town-name" aria-label="Town name" placeholder={world ? '' : 'Naming the town…'} bind:value={name} oninput={() => (named = true)} maxlength="24" />
          <button class="btn ghost roll" aria-label="Another name" title="Another name" onclick={rollName}><Icon name="icon.ui.dice" size={18} /></button>
        </div>
        {#if world?.world}
          {#key world.seed}<p class="lede">{world.world.tagline}</p>{/key}
        {/if}
      </div>

      <!-- The world's choices, in sections down the panel (scrolling when they don't fit). A switch
           between them and tools for the land itself can sit first in the panel, over the sections. -->
      <div class="panel" use:avoid>
        <div class="sections">
          <section class="group" aria-labelledby="place-head">
            <h2 class="eyebrow" id="place-head">Place</h2>
            <div class="places" role="radiogroup" aria-labelledby="place-head" aria-describedby="place-blurb" tabindex="-1" onkeydown={onPlaceKey}>
              {#each WORLD_KINDS as k (k)}
                <button class="place" role="radio" aria-checked={kind === k} class:on={kind === k} tabindex={kind === k ? 0 : -1} disabled={!templates} onclick={() => choose(k)}>
                  <span class="glyph">{@render glyph(k)}</span>
                  <span class="label">{WORLDS[k].label}</span>
                </button>
              {/each}
            </div>
            <p class="blurb" id="place-blurb">{WORLDS[kind].blurb}</p>
          </section>

          <section class="group" aria-labelledby="yours-head">
            <h2 class="eyebrow" id="yours-head">Your world</h2>
            <p class="hint summary">{sizeHint || '\u00a0'}</p>
            <div class="options">
              {#each OPTION_FIELDS as f (f.key)}
                <div class="option">
                  <span class="eyebrow sub">{f.label}</span>
                  <Segmented fill label={f.label} value={options[f.key]} options={f.choices} onchange={(v) => setOption(f.key, v)} />
                </div>
              {/each}
            </div>
          </section>

          <div class="game">
            <section class="group" aria-labelledby="game-head">
              <h2 class="eyebrow" id="game-head">Game</h2>
              <Segmented
                fill
                label="Game"
                bind:value={app.mode}
                options={[
                  { value: 'living', label: 'Living' },
                  { value: 'creative', label: 'Creative' },
                ]}
              />
              <p class="hint">{app.mode === 'creative' ? 'Building is free. No rent or bills at home.' : 'Your residents earn the money to build with.'}</p>
            </section>
            <section class="group" aria-labelledby="lifespan-head">
              <h2 class="eyebrow" id="lifespan-head">Lifespan</h2>
              <Segmented
                fill
                label="Lifespan"
                bind:value={app.lifespan}
                options={[
                  { value: 'off', label: 'Off' },
                  { value: 'short', label: 'Short' },
                  { value: 'normal', label: 'Normal' },
                  { value: 'long', label: 'Long' },
                ]}
              />
              <p class="hint">{lifespanHint(app.lifespan)}</p>
            </section>
          </div>
        </div>
      </div>
    </aside>

    <section class="view">
      {#if world}
        <TownStage {labels} {hover} selected={focus} {focus} onhover={(p) => (hover = p)} onselect={(p) => (focus = p === focus ? null : p)} />
      {:else}
        <div></div>
      {/if}
      {#if world}
        <div class="stats" use:avoid aria-label="World summary">
          <span><b>{roadCount}</b> {roadCount === 1 ? 'road' : 'roads'}</span>
          <span><b>{homes}</b> homes</span>
          {#if farms}<span><b>{farms}</b> {farms === 1 ? 'farm' : 'farms'}</span>{/if}
          <span><b>{places}</b> public {places === 1 ? 'place' : 'places'}</span>
        </div>
      {/if}
      <div class="actions" use:avoid>
        <button class="btn" disabled={!templates} onclick={another}><Icon name="icon.ui.dice" size={18} /> Another world</button>
        <button class="btn primary" disabled={!world || making || waiting} onclick={next}>Choose a household →</button>
      </div>
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
    </section>
  </div>
</div>

<style>
  /* Beside the town: one slim panel down the left (the world's name, its place and options, the
     game's), the town in the rest of the screen, the way on at its top right. */
  .screen {
    --pad: 18px;
    position: relative;
    /* (Viewport units count in the UI scale.) */
    height: calc(100vh / var(--ui-scale));
    container: setup / inline-size;
  }
  .layout {
    height: 100%;
    display: grid;
    grid-template-columns: clamp(300px, 24%, 340px) minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr);
    grid-template-areas: 'side view';
    padding: 20px 28px 24px;
    gap: 24px;
  }
  /* (As tall as its choices, and no taller than the screen: then they scroll.) */
  .side {
    grid-area: side;
    align-self: start;
    max-height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: rise 520ms var(--ease) both;
  }
  .head {
    grid-area: head;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 5px;
    padding: 10px var(--pad) 14px;
    border-bottom: 1px solid var(--hairline);
  }
  .back {
    height: 32px;
    padding: 0 10px;
    margin: 0 0 2px -10px;
    gap: 6px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .eyebrow {
    margin: 0;
    color: #4a5163;
  }
  /* The name is a field you can type in, not a heading: a surface and a hairline of its own at rest,
     a solid field under the pointer, the accent ring while it has the caret. The dice (roll another
     name) sits in the field's end. */
  .name-field {
    align-self: stretch;
    display: flex;
    align-items: center;
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
    height: 42px;
    padding: 0;
    font-size: 22px;
    font-weight: 750;
    letter-spacing: -0.02em;
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
  /* (Two lines kept for it, as for the place's blurb: the choices under it stay put from world to world.) */
  .lede {
    margin: 2px 0 0;
    min-height: 2lh;
    font-size: 13px;
    font-weight: 550;
    font-style: italic;
    line-height: 1.4;
    color: #3b4254;
    animation: rise 420ms var(--ease) both;
  }
  /* The choices: they scroll under the name when the window is too short for them all. */
  .panel {
    grid-area: panel;
    flex: 1;
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
  }
  .sections {
    display: flex;
    flex-direction: column;
  }
  .group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px var(--pad) 14px;
  }
  .group + .group,
  .group + .game,
  .game .group + .group {
    border-top: 1px solid var(--hairline);
  }
  .hint {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-muted);
  }
  /* The nine places: a glyph each with its name under it, the chosen one ringed. */
  .places {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }
  .place {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-width: 0;
    padding: 6px 4px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    background: color-mix(in srgb, var(--surface) 40%, transparent);
    color: var(--text-strong);
    transition:
      background var(--fast) var(--ease),
      border-color var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
  }
  .place:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.8);
  }
  .place:disabled {
    cursor: default;
  }
  .place.on {
    background: #fff;
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .glyph {
    display: grid;
    place-items: center;
    width: 50px;
    height: 30px;
    border-radius: var(--radius-sm);
    background: var(--surface-muted);
    color: #4a5163;
    transition:
      background var(--fast) var(--ease),
      color var(--fast) var(--ease);
  }
  .glyph :global(svg) {
    width: 38px;
    height: 25px;
  }
  .place.on .glyph {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .place .label {
    max-width: 100%;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .place.on .label {
    color: var(--accent);
  }
  /* (Two lines kept for it, the longest place's: the options under it don't jump as places change.) */
  .blurb {
    margin: 2px 0 0;
    min-height: 2lh;
    font-size: 12.5px;
    line-height: 1.4;
    font-weight: 550;
    color: var(--text-muted);
  }
  .summary {
    color: #3b4254;
    font-weight: 550;
    margin-bottom: 2px;
  }
  .options {
    display: grid;
    gap: 8px;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 13px;
  }
  .eyebrow.sub {
    font-size: 10.5px;
    color: var(--text-muted);
  }
  .group :global(.segmented) {
    font-size: 13px;
  }
  .view {
    grid-area: view;
    position: relative;
    min-height: 0;
    display: grid;
    grid-template-rows: minmax(0, 1fr);
  }
  /* The way on, at the top right over the town, a short haze behind it so it reads on roofs and roads. */
  .actions {
    position: absolute;
    top: 0;
    right: 0;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
    animation: rise 520ms 80ms var(--ease) both;
  }
  .actions::before {
    content: '';
    position: absolute;
    inset: -60px -60px -50px -90px;
    z-index: -1;
    background: radial-gradient(closest-side, rgba(252, 249, 243, 0.7), rgba(252, 249, 243, 0.35) 55%, rgba(252, 249, 243, 0));
    pointer-events: none;
  }
  /* The world in figures, small at the top left of the town (clear of the stream pill at the bottom). */
  .stats {
    position: absolute;
    top: 6px;
    left: 0;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    max-width: calc(100% - 420px);
    color: #3b4254;
    font-size: 12px;
    font-weight: 550;
    pointer-events: none;
    animation: rise 520ms 120ms var(--ease) both;
  }
  .stats span {
    padding: 5px 10px;
    border-radius: var(--radius-pill);
    background: var(--glass-menu);
    border: 1px solid var(--glass-menu-border);
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(12px) saturate(1.4);
    -webkit-backdrop-filter: blur(12px) saturate(1.4);
  }
  .stats b {
    color: var(--text-strong);
    font-weight: 750;
  }
  /* The world's making, step by step: in the middle of the town's view (over the last world, if any, which stays in view meanwhile). */
  .making {
    position: absolute;
    inset: 0;
    margin: auto;
    width: min(320px, 100%);
    height: max-content;
    z-index: 2;
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
  /* A window not much wider than the panel and the town: the stream pill, centred at the window's
     bottom edge, would reach the panel's end; the panel stops short of it (its choices scroll sooner),
     and the stats make way for the buttons. */
  @container setup (max-width: 1000px) {
    .side {
      max-height: calc(100% - 72px / var(--ui-scale));
    }
  }
  @container setup (max-width: 1120px) {
    .stats {
      display: none;
    }
  }
  /* Too narrow for the town beside the panel (a small window or a large UI scale): the name on top,
     the way on under it, the town, then the choices in a panel of their own. The stats are left
     out (the world's line sums them up). */
  @container setup (max-width: 760px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: auto minmax(0, 1fr) minmax(0, 1fr);
      grid-template-areas: 'head' 'view' 'panel';
      gap: 12px;
      /* (The stream pill, centred at the window's bottom edge, gets a strip of town of its own under the panel.) */
      padding: 16px 20px calc(70px / var(--ui-scale) + 26px);
    }
    .side {
      display: contents;
    }
    .head,
    .panel {
      background: var(--glass-menu);
      border: 1px solid var(--glass-menu-border);
      border-radius: var(--radius-lg);
      box-shadow: var(--glass-menu-shadow);
      backdrop-filter: blur(22px) saturate(1.5);
      -webkit-backdrop-filter: blur(22px) saturate(1.5);
    }
    .head {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr);
      align-items: center;
      gap: 6px 12px;
      padding: 10px 14px 12px;
    }
    .back {
      margin: 0 0 0 -6px;
    }
    .head > :not(.back) {
      grid-column: 1 / -1;
    }
    .head > .eyebrow {
      grid-column: 2;
      grid-row: 1;
    }
    .view {
      grid-template-rows: auto minmax(0, 1fr);
      gap: 10px;
    }
    /* (First in the view, over the town: before it, or its stand-in while the first world is made.) */
    .actions {
      order: -1;
      position: relative;
      justify-content: flex-start;
    }
    .actions::before {
      display: none;
    }
    /* (All nine in a row at the narrow layout's widest; a second row as it narrows.) */
    .places {
      grid-template-columns: repeat(auto-fill, minmax(68px, 1fr));
      gap: 4px;
    }
    .place {
      padding-inline: 0;
    }
    .place .label {
      font-size: 11px;
      letter-spacing: -0.01em;
    }
    .lede {
      min-height: 0;
    }
    .options {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px 16px;
    }
    .game {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .game .group + .group {
      border-top: 0;
      border-left: 1px solid var(--hairline);
    }
  }
  /* Narrower still (a phone's width): one option to a row. */
  @container setup (max-width: 560px) {
    .options,
    .game {
      grid-template-columns: minmax(0, 1fr);
    }
    .game .group + .group {
      border-left: 0;
      border-top: 1px solid var(--hairline);
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
