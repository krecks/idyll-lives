<script lang="ts">
  /**
   * New game, step 1: the land. A place, its size, how hilly and wooded it is, and its seed (the
   * world's code: typed, pasted, copied or rolled), made with no town on it (the town job's `land`);
   * then shaped by hand if the player likes (Shape the land). Its streets are laid out on it in step 2.
   */
  import { onMount } from 'svelte';
  import { CODE_FLAG, WORLD_FLAG } from '../../debug/benchFlags';
  import { menuScene } from '../../game/menuScene';
  import type { NeighbourhoodDraft } from '../../game/town';
  import { Superseded, townJob } from '../../game/townJobs';
  import { WORLD_KINDS, WORLDS, type WorldKind } from '../../game/world';
  import { writeCode } from '../../game/worldCode';
  import { OPTION_FIELDS } from '../../game/worldOptions';
  import type { TownBuildStep } from '../../render/types';
  import { app } from '../app.svelte';
  import { toast } from '../state.svelte';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import TownStage, { type StageTool } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import { applyLand, builtOf, LandEditor } from '../../game/landEdit';
  import LandPalette from '../land/LandPalette.svelte';
  import { installLandHook } from '../land/landHook';
  import { LandStroke, type LandSurface } from '../land/landStroke';
  import { brushOf, startBrush, type BrushState } from '../land/landTools';
  import { codeFrom, firstCode, fitted, landKey, landStats, optionsOf, SIZE_CHOICES, startCode, stepLabel, STEPS as FLOW, withSeed, type Chosen } from '../newGame';
  import SetupConfirm from './SetupConfirm.svelte';
  import SetupShell, { type Making } from './SetupShell.svelte';

  const { content } = services;
  /** A new seed: a different land every time. */
  const roll = () => (Math.random() * 2 ** 31) >>> 0;

  // The code: as chosen before (back from step 2), else the page's (`?code=`, `?world=`, `?options=`: the timing tools' worlds), else the countryside's.
  app.code ??= firstCode(CODE_FLAG, WORLD_FLAG, roll());
  const code = $derived(app.code ?? startCode('country', 0));
  /** The land shown: step 1's from before (back from step 2), else the one made now. */
  let draft = $state.raw<NeighbourhoodDraft | null>(app.land?.draft ?? null);

  /** The land's own options besides its size (the town's are step 2's). */
  const LAND_FIELDS = OPTION_FIELDS.filter((f) => f.key === 'hills' || f.key === 'woods') as readonly { key: 'hills' | 'woods'; label: string; choices: readonly { value: string; label: string }[] }[];
  const sizes = SIZE_CHOICES.map((c) => ({ value: c.value as string, label: c.label }));

  /**
   * Making the land, as the progress panel lists it: made in the town worker, then built in 3D
   * (`TownBuildStep`, side by side: they tick off in any order).
   */
  const STEPS: readonly { key: 'land' | TownBuildStep; label: string }[] = [
    { key: 'land', label: 'Shaping the land' },
    { key: 'hills', label: 'Raising the hills' },
    { key: 'woods', label: 'Planting the woods' },
    { key: 'grass', label: 'Sowing the grass' },
  ];

  /** The land being made (in the town worker, then built); a newer one replaces it. */
  let job: AbortController | null = null;
  /** The progress panel's title while a land is made or built (null: none is). */
  let making = $state<string | null>(null);
  /** An option changed: the land is made again once they settle (`remake`); meanwhile it can't be taken on. */
  let settling: ReturnType<typeof setTimeout> | undefined;
  let waiting = $state(false);
  let done = $state.raw<ReadonlySet<string>>(new Set());
  const progress = $derived<Making | null>(making ? { title: making, steps: STEPS, done } : null);
  /** A part done of the land `control` makes (none once a newer one replaced it). */
  const tick = (control: AbortController) => (step: string) => {
    if (job === control) done = new Set([...done, step]);
  };

  /** The land for the code as it is now, made and built; the one being made meanwhile is dropped. */
  async function make() {
    clearTimeout(settling);
    waiting = false;
    job?.abort();
    const control = (job = new AbortController());
    const asked = code;
    // (Nothing in view yet: the first land is built in longer slices.)
    const first = !draft;
    making = first ? 'A new land' : 'Another land';
    done = new Set();
    const step = tick(control);
    try {
      const made = await townJob(content, { kind: 'land', world: asked.kind, seed: asked.seed, options: optionsOf(asked) }, control.signal, step);
      if (job !== control) return;
      step('land');
      // (Another land: it is surveyed again for the brushes, below.)
      dropEditor();
      menuScene.prepare(made.draft, made.preview);
      draft = made.draft;
      // (The panel stays until it is built and on screen.)
      await (shown = menuScene.show(made.draft, null, step, first));
    } catch (err) {
      if (err instanceof Superseded) return;
      console.error('[land] making the land failed:', err);
      toast('That land could not be made. Try another.');
    } finally {
      if (job === control) {
        job = null;
        making = null;
        if (view === 'shape') void survey();
      }
    }
  }
  $effect(() => () => {
    clearTimeout(settling);
    job?.abort();
  });

  /** `next` taken on as the code, and its land made (after `wait` ms if the options are being changed: a click or an arrow key each changes one, only the last is made). */
  function remake(next: Chosen, wait = 0) {
    app.code = next;
    clearTimeout(settling);
    if (!wait) return void make();
    waiting = true;
    settling = setTimeout(() => void make(), wait);
  }

  /** Asked first if the land is shaped (the shaping goes with a land made again), where it was asked for. */
  function unlessShaped(text: string, yes: string, at: Asked, go: () => void) {
    if (shaped) confirming = { text, yes, at, go };
    else go();
  }

  /** Another land of the place and options as they are (a new seed). */
  function another() {
    unlessShaped('Another land leaves your changes to this one behind.', 'Another land', 'code', () => remake(withSeed(code, roll())));
  }

  /** Another place: a new land there, with the place's own options. */
  function choose(k: WorldKind) {
    if (k === code.kind && draft) return;
    unlessShaped(`${WORLDS[k].label} is another land: your changes to this one go.`, `Go to ${WORLDS[k].label}`, 'panel', () => remake(startCode(k, roll())));
  }

  /** An option of the land's changed: the same seed made again with it (once the options settle). */
  function setOption(key: 'size' | 'hills' | 'woods', value: string) {
    if (code[key] === value) return;
    const label = key === 'size' ? 'size' : (LAND_FIELDS.find((f) => f.key === key)?.label.toLowerCase() ?? key);
    unlessShaped(`Changing the ${label} makes the land again: your changes to it go.`, `Change the ${label}`, 'panel', () => {
      // (A smaller land may not hold the town and villages chosen: then the nearest that fit, `fitted`.)
      const next = { ...code, [key]: value } as Chosen;
      remake(key === 'size' ? fitted(next) : next, 300);
    });
  }

  // ---- The world's code ------------------------------------------------------------------------

  /** The code field as the player is typing it (null: the code as it stands). */
  let typed = $state<string | null>(null);
  /** What was typed isn't a code. */
  let bad = $state(false);
  const written = $derived(writeCode(code));

  /** The typed code taken on (Enter, or the field left): its land made, unless it is the land shown. */
  function takeCode() {
    const text = typed?.trim();
    if (typed === null) return;
    const next = text ? codeFrom(text, code) : null;
    if (!next) {
      bad = !!text;
      if (!bad) typed = null;
      return;
    }
    typed = null;
    bad = false;
    if (writeCode(next) === written) return;
    // (A code that changes only the town's choices leaves the land as it is: step 2 lays them.)
    if (landKey(next, null) === landKey(code, null)) return void (app.code = next);
    unlessShaped('This code makes another land: your changes to this one go.', 'Use the code', 'code', () => remake(next));
  }

  function onCodeKey(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      takeCode();
    } else if (e.key === 'Escape') {
      typed = null;
      bad = false;
    }
  }

  async function copyCode() {
    try {
      await navigator.clipboard.writeText(written);
      toast(`Copied ${written}`);
    } catch {
      toast("The code couldn't be copied: select it and copy it by hand.");
    }
  }

  // ---- Shaping the land (the Shape panel) ------------------------------------------------------

  /** The panel shows the land's choices, or the land brushes. */
  let view = $state<'land' | 'shape'>('land');
  /** The brushes' editor, for the land shown (null until Shape is first chosen for it). */
  let editor = $state.raw<LandEditor | null>(null);
  let editorOf: NeighbourhoodDraft | null = null;
  let surveying = $state(false);
  let brush = $state<BrushState>(startBrush());
  /** Counts the land's changes (the editor changes it in place): what reads the editor reads it. */
  let landTick = $state(0);
  const land = $derived.by(() => {
    void landTick;
    const e = editor;
    const changed = e?.changed();
    return { canUndo: !!e?.canUndo, canRedo: !!e?.canRedo, hint: e?.hint ?? null, edited: !!e?.land.edited, changed: !!changed && (changed.heights || changed.water || changed.cover) };
  });
  /** The land differs from the land as generated (shaped here, or before step 2): what makes it again asks first. */
  const shaped = $derived(land.edited || (!editor && !!draft?.open?.land?.edited));
  /** The land in figures (the brushes' field once there is one: it follows the strokes). */
  const stats = $derived.by(() => {
    void landTick;
    return draft ? landStats(draft, editor && editorOf === draft ? editor.land : null) : null;
  });
  /** A mountain's ghost while it is dragged out (SVG path data, screen pixels). */
  let ghost = $state('');
  /** Where a question is asked: by the code field, over the panel's choices, or under the way on. */
  type Asked = 'code' | 'panel' | 'actions';
  /** A choice that would throw the land's shaping (or the town's changes) away, waiting for a yes (shown where it was made). */
  let confirming = $state<{ text: string; yes: string; keep?: string; at: Asked; go: () => void } | null>(null);
  /** Bumped when a choice is not made after all (the switches show the choice as it was). */
  let unasked = $state(0);
  /** The land on screen, once it is (it is surveyed from it). */
  let shown: Promise<void> = Promise.resolve();

  function answer(yes: boolean) {
    const c = confirming;
    confirming = null;
    if (yes) c?.go();
    else unasked++;
  }

  const surface: LandSurface = {
    pick: (x, y) => menuScene.landAt(x, y),
    changed: (rect, done) => menuScene.landChanged(rect, done),
    cursor: (c) => menuScene.brush(c),
    project: (x, y, z, out) => menuScene.project(x, y, z, out),
  };
  const stroke = $derived(editor ? new LandStroke(editor, surface, { brush: () => brushOf(brush), ended: () => landTick++, ghost: (d) => (ghost = d) }) : null);
  /** The brush in the stage's hand while Shape is up (and the land isn't being made again). */
  const stageTool = $derived.by((): StageTool | null => {
    const s = stroke;
    if (view !== 'shape' || !s || making) return null;
    return {
      down: (x, y) => s.down(x, y),
      move: (x, y) => s.move(x, y),
      up: () => void s.up(),
      hover: (x, y) => s.hover(x, y),
      leave: () => s.leave(),
      cancel: () => {
        s.cancel();
        ghost = '';
        landTick++;
      },
    };
  });

  function setView(v: 'land' | 'shape') {
    view = v;
    confirming = null;
    if (v === 'shape') void survey();
  }

  /** The land shown, for the brushes: surveyed once (behind a spinner), then drawn from the field the brushes change. */
  async function survey() {
    const w = draft;
    if (!w || making || (editor && editorOf === w) || surveying) return;
    surveying = true;
    try {
      await shown;
      const field = await menuScene.landOf();
      const preview = menuScene.preview;
      if (draft !== w || making || !preview) return;
      // (A land shaped before step 2 is shaped already: what makes it again still asks.)
      field.edited ||= !!w.open?.land?.edited;
      editor = LandEditor.forWorld(field, builtOf(preview), { water: w.open?.water, zones: w.open?.zones });
      editorOf = w;
      menuScene.setLand(field);
      landTick++;
    } catch (err) {
      console.error('[land] surveying the land failed:', err);
      toast("The land couldn't be surveyed. Try again in a moment.");
      view = 'land';
    } finally {
      surveying = false;
    }
  }

  /** The editor let go (another land: it is surveyed again). */
  function dropEditor() {
    stroke?.dispose();
    editor = null;
    editorOf = null;
    ghost = '';
    landTick++;
  }

  function landUndo() {
    const patch = editor?.undo();
    if (patch) menuScene.landChanged(patch.rect, true);
    landTick++;
  }
  function landRedo() {
    const patch = editor?.redo();
    if (patch) menuScene.landChanged(patch.rect, true);
    landTick++;
  }
  function landReset() {
    const rect = editor?.reset();
    if (rect) menuScene.landChanged(rect, true);
    landTick++;
  }

  function onShapeKey(e: KeyboardEvent) {
    if (view !== 'shape' || !editor || making) return;
    const t = e.target;
    if (t instanceof HTMLInputElement && t.type !== 'range') return;
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
    const key = e.key.toLowerCase();
    if (key === 'z') {
      e.preventDefault();
      if (e.shiftKey) landRedo();
      else landUndo();
    } else if (key === 'y' && e.ctrlKey) {
      e.preventDefault();
      landRedo();
    }
  }

  /**
   * On to step 2 with the land as shaped (`applyLand`: in its town file, the water walked round, the
   * zones). The same land as before (back and forth, nothing changed) keeps its town; another one
   * lays the town again there, asked first if the town has the player's own changes.
   */
  function next() {
    if (!draft) return;
    stroke?.cancel();
    const editing = editor && editorOf === draft ? editor : null;
    const chosen = editing ? applyLand(draft, editing) : draft;
    const key = landKey(code, chosen.open?.land);
    const go = () => {
      if (app.land?.key !== key) {
        // (Water shaped by hand, now or before step 2, leaves the town laid on it to find its own bridges.)
        const keptBefore = draft === app.land?.draft ? app.land.waterKept : true;
        app.land = { draft: chosen, key, waterKept: keptBefore && !editing?.changed().water };
        app.site = null;
      }
      app.screen = 'streets';
    };
    if ((app.layout?.edited || app.world?.edited) && app.land && app.land.key !== key) {
      confirming = { text: 'The land changed: the streets are laid out on it again, and your changes to the layout and the town go.', yes: 'Lay it out again', keep: 'Keep my town', at: 'actions', go };
      return;
    }
    go();
  }

  onMount(async () => {
    if (!draft) {
      // (Not the town the main menu glided away from, if it is still there.)
      void menuScene.hide();
      await make();
      return;
    }
    // Back from step 2: the land as it was taken on (its town goes from view meanwhile).
    const control = (job = new AbortController());
    making = 'The land';
    done = new Set(['land']);
    try {
      await (shown = menuScene.show(draft, null, tick(control)));
    } catch (err) {
      console.warn('[land] showing the land failed', err);
    } finally {
      if (job === control) {
        job = null;
        making = null;
      }
    }
  });

  /** Leaving for the main menu: its picture is taken of the world in view (`App.svelte`). */
  function toMenu() {
    clearTimeout(settling);
    job?.abort();
    app.toMenu();
  }

  // No lots to light up on a bare land.
  $effect(() => {
    menuScene.highlight({});
    return () => menuScene.highlight({});
  });
  // Leaving step 1: the brush put down, its cursor gone.
  $effect(() => () => {
    stroke?.dispose();
    menuScene.brush(null);
  });
  $effect(() =>
    installLandHook({
      editor: () => editor,
      surface: () => surface,
      undo: landUndo,
      redo: landRedo,
      changed: () => landTick++,
      camera: { pan: (dx, dz) => menuScene.pan(dx, dz), zoom: (f) => menuScene.nudge(0, f) },
    }),
  );

  /**
   * Arrow keys in the place grid: across by one (wrapping round), up and down by a row, as many as
   * the grid shows in a row (three beside the land, more under it when narrow). A radio group: a move
   * chooses the place, as a click does.
   */
  function onPlaceKey(e: KeyboardEvent) {
    const grid = e.currentTarget as HTMLElement;
    const row = getComputedStyle(grid).gridTemplateColumns.split(' ').length;
    const step = ({ ArrowRight: 1, ArrowLeft: -1, ArrowDown: row, ArrowUp: -row } as Record<string, number>)[e.key];
    if (!step) return;
    e.preventDefault();
    const n = WORLD_KINDS.length;
    const at = WORLD_KINDS.indexOf(code.kind);
    const to = Math.abs(step) === 1 ? (at + step + n) % n : at + step;
    if (to < 0 || to >= n) return;
    choose(WORLD_KINDS[to]);
    (grid.children[to] as HTMLElement | undefined)?.focus();
  }

  /** A share as a whole percent (a little, but not none: under 1%). */
  const percent = (share: number) => (share > 0 && share < 0.005 ? '<1%' : `${Math.round(share * 100)}%`);
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

{#snippet ask(at: Asked)}
  {#if confirming?.at === at}
    <SetupConfirm label="Your changes" text={confirming.text} yes={confirming.yes} no={confirming.keep ?? 'Keep my land'} onanswer={answer} />
  {/if}
{/snippet}

{#snippet head()}
  <span class="eyebrow sub code-label" id="code-label">World code</span>
  <div class="name-field code-field" class:bad>
    <input
      class="code"
      aria-labelledby="code-label"
      aria-invalid={bad}
      aria-describedby="code-hint"
      spellcheck="false"
      autocomplete="off"
      value={typed ?? written}
      oninput={(e) => ((typed = e.currentTarget.value), (bad = false))}
      onchange={takeCode}
      onkeydown={onCodeKey}
    />
    <button class="btn ghost copy" title="Copy the code" onclick={copyCode}>Copy</button>
    <button class="btn ghost roll" aria-label="Another land" title="Another land" onclick={another}><Icon name="icon.ui.dice" size={18} /></button>
  </div>
  <p class="hint" id="code-hint">
    {#if bad}Not a code: type a number, or paste a code such as <b>river-mhs-k2x9q</b>.{:else if app.beyondCode}A number or a code makes the world as generated, and its people. Your own streets, zones and changes are not in the code.{:else}A number or a code makes the world as generated, and its people. Your own changes to the land and the town are not in it.{/if}
  </p>
  {@render ask('code')}
  {#if draft?.world}
    {#key draft.seed}<p class="lede">{draft.world.tagline}</p>{/key}
  {/if}
{/snippet}

{#snippet panel()}
  <!-- The land's choices, in sections down the panel, or (Shape) the brushes for the land itself; a switch between the two first. -->
  <div class="switch">
    <Segmented
      fill
      label="Panel"
      value={view}
      options={[
        { value: 'land', label: 'Land' },
        { value: 'shape', label: 'Shape the land', disabled: !draft },
      ]}
      onchange={setView}
    />
  </div>
  {@render ask('panel')}
  {#if view === 'shape'}
    <div class="shape">
      {#if surveying || (!editor && making)}
        <p class="surveying" role="status"><span class="spinner"></span> {making ? 'Making the land…' : 'Surveying the land…'}</p>
      {:else if editor}
        <LandPalette bind:brush canUndo={land.canUndo} canRedo={land.canRedo} canReset={land.changed} hint={land.hint} disabled={!!making} onundo={landUndo} onredo={landRedo} onreset={landReset} />
        <p class="keys"><b>Left</b> paints · <b>right</b> turns · <b>middle</b> or <kbd>Space</kbd> pans · wheel zooms</p>
      {/if}
    </div>
  {:else}
    <section class="group" aria-labelledby="place-head">
      <h2 class="eyebrow" id="place-head">Place</h2>
      <div class="places" role="radiogroup" aria-labelledby="place-head" aria-describedby="place-blurb" tabindex="-1" onkeydown={onPlaceKey}>
        {#each WORLD_KINDS as k (k)}
          <button class="place" role="radio" aria-checked={code.kind === k} class:on={code.kind === k} tabindex={code.kind === k ? 0 : -1} onclick={() => choose(k)}>
            <span class="glyph">{@render glyph(k)}</span>
            <span class="label">{WORLDS[k].label}</span>
          </button>
        {/each}
      </div>
      <p class="blurb" id="place-blurb">{WORLDS[code.kind].blurb}</p>
    </section>

    <section class="group" aria-labelledby="land-head">
      <h2 class="eyebrow" id="land-head">Your land</h2>
      <div class="options">
        {#key unasked}
          <div class="option">
            <span class="eyebrow sub">Size</span>
            <Segmented fill label="Size" value={code.size as string} options={sizes} onchange={(v) => setOption('size', v)} />
          </div>
          {#each LAND_FIELDS as f (f.key)}
            <div class="option">
              <span class="eyebrow sub">{f.label}</span>
              <Segmented fill label={f.label} value={code[f.key] as string} options={f.choices} onchange={(v) => setOption(f.key, v)} />
            </div>
          {/each}
        {/key}
      </div>
      <p class="hint">The town and its villages are laid on it in the next step.</p>
    </section>
  {/if}
{/snippet}

{#snippet stage()}
  {#if draft}
    <TownStage labels={[]} shot="world" tool={stageTool} {ghost} />
  {:else}
    <div></div>
  {/if}
{/snippet}

{#snippet figures()}
  {#if stats}
    <span><b>{stats.width} × {stats.depth}</b> m</span>
    <span><b>{percent(stats.water)}</b> water</span>
    <span><b>{percent(stats.woods)}</b> woods</span>
  {/if}
{/snippet}

{#snippet actions()}
  <button class="btn primary" disabled={!draft || !!making || waiting} onclick={next}>{FLOW[0].next}</button>
  {@render ask('actions')}
{/snippet}

<svelte:window onkeydown={onShapeKey} />

<SetupShell label="A new land" step={stepLabel(1)} back={{ label: 'Menu', go: toMenu }} {head} {panel} {stage} stats={figures} {actions} making={progress} />

<style>
  .code-label {
    margin-top: 4px;
  }
  .code-field.bad {
    border-color: var(--bad);
  }
  /* (Over the frame's own field type: a code is read a character at a time.) */
  .name-field.code-field .code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 16px;
    font-weight: 650;
    letter-spacing: 0;
  }
  .code-field .copy {
    flex: none;
    height: 34px;
    padding: 0 8px;
    font-size: 12.5px;
    font-weight: 650;
    color: #4a5163;
  }
  .hint b {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-weight: 650;
  }
  /* Land | Shape the land: over everything else in the panel. */
  .switch {
    position: sticky;
    top: 0;
    z-index: 1;
    padding: 12px var(--pad) 10px;
    background: linear-gradient(var(--glass-menu) 75%, transparent);
  }
  .switch :global(.segmented) {
    font-size: 13px;
  }
  .shape {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 4px var(--pad) 16px;
    animation: rise 260ms var(--ease) both;
  }
  .surveying {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 18px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .keys {
    margin: 0;
    font-size: 11.5px;
    line-height: 1.5;
    color: var(--text-muted);
  }
  .keys b {
    font-weight: 650;
    color: var(--text);
  }
  kbd {
    padding: 0 5px;
    border-radius: 4px;
    background: var(--hairline);
    font-family: inherit;
    font-size: 11px;
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
  .place:hover {
    background: rgba(255, 255, 255, 0.8);
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
  /* Too narrow for the land beside the panel: all nine places in a row at the narrow layout's widest, a second row as it narrows. */
  @container setup (max-width: 760px) {
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
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
