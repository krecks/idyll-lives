<script lang="ts">
  /**
   * New game, step 2: the streets and zones (the layout), on step 1's land. It opens on the proposed
   * layout (the `layout` job: the generator's streets with its town painted as zones, no buildings
   * yet), for the player to draw streets on and paint (the editor's `LAYOUT_TOOLS`: the zone brush
   * where step 3 has the Lot tool). Its name, how big the town is and how many villages lie round it
   * (choices the land is too small for are ruled out), another proposal from a new seed, Clear all
   * (bare land), Clear zones, and the town moved by a click on the land: each proposes the layout
   * again, asked first once the player has changed it (`app.layout.edited`) or tweaked the town
   * built from it (`app.world.edited`).
   *
   * The way on (Build) waits for the layout to be settled and checked, and is shut while a problem
   * with the streets blocks it or the zones would build fewer than two homes. A layout changed since
   * the town was built builds it again there, asked first if that town was tweaked.
   */
  import { onMount, untrack } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import { OVERVIEW_CLOSE } from '../../render/shared/overviewCamera';
  import type { Pt } from '../../game/roads';
  import type { NeighbourhoodDraft } from '../../game/town';
  import { keepHouses, Superseded, townJob } from '../../game/townJobs';
  import { LAND_SIZE, specOf, townFits, worldName, type WorldStep } from '../../game/world';
  import { OPTION_FIELDS } from '../../game/worldOptions';
  import type { TownBuildStep } from '../../render/types';
  import { app } from '../app.svelte';
  import { toast } from '../state.svelte';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import TownStage, { type LotLabel, type StageTool } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import { changesWord, fitted, goesWords, optionsOf, SIZE_CHOICES, startCode, stepLabel, STEPS, townKey, whatGoes, worldKeyOf, type Chosen } from '../newGame';
  import SetupConfirm from './SetupConfirm.svelte';
  import SetupShell, { type Making } from './SetupShell.svelte';
  import { TownEditorState } from '../townedit/editor.svelte';
  import { clearAll, clearZones, summaryLine, usesSummary } from '../townedit/layoutApi';
  import TownEditor from '../townedit/TownEditor.svelte';

  const { content } = services;
  /** A new seed: a different town every time. */
  const roll = () => (Math.random() * 2 ** 31) >>> 0;

  const code = $derived(app.code ?? startCode('country', 0));
  /** The layout shown (its streets settled on the land): from before, else the one proposed now. */
  const town = $derived(app.layout?.draft ?? null);
  /** The town's name as typed or rolled here, else the one it was laid with. */
  let name = $state(app.townName ?? app.layout?.draft.name ?? '');
  let hover = $state<number | null>(null);
  let focus = $state<number | null>(null);

  /** The layout editor (its tools, cards, undo and Problems list), over the layout shown. */
  const editor = new TownEditorState({
    content,
    step: 'layout',
    getName: () => name,
    setName(typed) {
      name = typed;
      app.townName = typed.trim() || null;
    },
    glide: (plot) => (focus = plot),
  });
  // (The editor's own writes to `app.layout` come back here: it keeps its session over them.)
  $effect(() => {
    const l = app.layout;
    untrack(() => editor.attach(making ? null : l));
  });
  $effect(() => () => editor.dispose());
  /** The layout as the edits left it (the figures follow the edits at once). */
  const plan = $derived(editor.ready ? editor.draft : town);
  /** On to step 3 asked for: waiting for the layout to be settled and checked. */
  let leaving = $state(false);
  const blocked = $derived(editor.issues.blocking.length > 0);

  /** What the zones would build, about (`usesSummary`), and in a line. */
  const zoned = $derived(plan ? usesSummary(plan) : null);
  /** Fewer than two homes to come: those the zones would build, about, and any lots the layout has already. */
  const fewHomes = $derived(!!zoned && !!plan && zoned.homes + plan.slots.filter((s) => s.kind === 'house').length < 2);

  /** The town's own options (the land's are step 1's). */
  const TOWN_FIELDS = OPTION_FIELDS.filter((f) => f.key === 'town' || f.key === 'villages') as readonly { key: 'town' | 'villages'; label: string; choices: readonly { value: string; label: string }[] }[];
  /** A choice the land is too small for, ruled out: a town too big even with no villages, villages too many for the town chosen. */
  const fits = (key: 'town' | 'villages', value: string) =>
    key === 'town' ? townFits(code.size, value as Chosen['town'], 'none') : townFits(code.size, code.town, value as Chosen['villages']);
  /** What the land has no room for, said in a line ("a city or many villages"); empty: room for every choice. */
  const ruledOut = $derived(
    TOWN_FIELDS.flatMap((f) => f.choices.filter((c) => !fits(f.key, c.value)).map((c) => (f.key === 'town' ? `a ${c.label.toLowerCase()}` : `${c.label.toLowerCase()} villages`))).join(' or '),
  );
  const sizeLabel = $derived(SIZE_CHOICES.find((s) => s.value === code.size)?.label.toLowerCase() ?? code.size);

  const roadCount = $derived(plan ? plan.roads.filter((r) => r.kind !== 'trail').length : 0);
  const villages = $derived(plan?.open ? Math.max(0, plan.open.areas.length - 1) : 0);

  // Public places and farms have their names on them (a layout proposed has none: its places are still zones).
  const labels = $derived.by((): LotLabel[] => {
    const w = town;
    if (!w || moving) return [];
    return w.slots.filter((s) => s.kind !== 'house' || s.lot === 'farm').map((s) => ({ plot: s.index, title: s.name, kind: s.kind === 'house' ? 'sale' : 'park' }));
  });

  /** Laying out the streets, as the progress panel lists it: in the town worker (`WorldStep`, in turn), then built in 3D (`TownBuildStep`). */
  const MAKING: readonly { key: WorldStep | TownBuildStep; label: string }[] = [
    { key: 'roads', label: 'Laying the roads' },
    { key: 'homes', label: 'Placing the homes' },
    { key: 'houses', label: 'Building the houses' },
    { key: 'hills', label: 'Settling the land' },
    { key: 'woods', label: 'Planting the woods' },
    { key: 'town', label: 'Building the town' },
    { key: 'grass', label: 'Sowing the grass' },
  ];

  /** The layout being proposed (in the town worker, then shown); a newer one replaces it. */
  let job: AbortController | null = null;
  /** The progress panel's title while a layout is proposed or shown (null: none is). */
  let making = $state<string | null>(null);
  /** An option changed: the layout is proposed again once they settle; meanwhile it can't be taken on. */
  let settling: ReturnType<typeof setTimeout> | undefined;
  let waiting = $state(false);
  let done = $state.raw<ReadonlySet<string>>(new Set());
  const progress = $derived<Making | null>(making ? { title: making, steps: MAKING, done } : null);
  const tick = (control: AbortController) => (step: string) => {
    if (job === control) done = new Set([...done, step]);
  };

  /** The layout proposed for the land and the code as they are now, and shown; the one being proposed meanwhile is dropped. */
  async function propose() {
    clearTimeout(settling);
    waiting = false;
    const land = app.land;
    if (!land) return;
    job?.abort();
    const control = (job = new AbortController());
    // (The editor lets go of the layout being replaced: no edit of it is settled over the new one.)
    editor.attach(null);
    const asked = code;
    const site = app.site;
    const key = townKey(land.key, asked, site);
    making = app.layout ? 'Another layout' : 'Laying out the streets';
    done = new Set();
    const step = tick(control);
    try {
      const made = await townJob(
        content,
        { kind: 'layout', land: land.draft, seed: asked.townSeed ?? asked.seed, name: app.townName ?? undefined, options: optionsOf(asked), ...(site ? { site } : {}), waterKept: land.waterKept },
        control.signal,
        step,
      );
      if (job !== control) return;
      // Its houses and preview come with it: the menu draws it without making them again.
      keepHouses(made);
      menuScene.prepare(made.draft, made.preview);
      app.layout = { draft: made.draft, key, proposal: made.proposal, edited: false, plan: made.plan ?? made.proposal.plan, ...(made.marks ? { marks: made.marks } : {}) };
      // (A new proposal builds its town from the build's first seed: the code's own town.)
      app.buildSeed = 0;
      cleared = false;
      name = app.townName ?? made.draft.name;
      focus = null;
      hover = null;
      // (The panel stays until it is built and on screen.)
      await menuScene.show(made.draft, null, step);
    } catch (err) {
      if (err instanceof Superseded) return;
      console.error('[layout] laying out the streets failed:', err);
      // (On a shaped land the generator says why it couldn't: no dry land big enough.)
      toast(err instanceof Error && err.message && app.land?.draft.open?.land?.edited ? err.message : 'The streets could not be laid out. Try another layout.');
    } finally {
      if (job === control) {
        job = null;
        making = null;
        editor.attach(app.layout);
      }
    }
  }
  $effect(() => () => {
    clearTimeout(settling);
    job?.abort();
  });

  /** What proposing again asks about (`whatGoes`): the layout's edits, the built town's tweaks. */
  const asks = () => whatGoes('propose', { layoutEdited: !!app.layout?.edited, townTweaked: !!app.world?.edited }).ask;
  /** Asked first if the layout has the player's own changes, or the town built from it was tweaked (proposing again throws them away). */
  function unlessEdited(text: string, yes: string, go: () => void) {
    if (asks().length) confirming = { text, yes, no: 'Keep my layout', go };
    else go();
  }
  let confirming = $state<{ text: string; yes: string; no: string; go: () => void } | null>(null);
  /** Bumped when a choice is not made after all (the switches show the choice as it was). */
  let unasked = $state(0);
  function answer(yes: boolean) {
    const c = confirming;
    confirming = null;
    if (yes) c?.go();
    else unasked++;
  }
  /** What proposing again throws away, said for the question. */
  const yourWork = () => `${goesWords(asks())} go`;

  /** The town or the villages changed: the layout proposed again with them, on the same seed (once the options have stood for 300 ms). */
  function setOption(key: 'town' | 'villages', value: string) {
    if (code[key] === value) return;
    const label = key === 'town' ? 'town' : 'villages';
    unlessEdited(`Changing the ${label} lays out the streets again: ${yourWork()}.`, `Change the ${label}`, () => {
      app.code = fitted({ ...code, [key]: value } as Chosen);
      clearTimeout(settling);
      waiting = true;
      settling = setTimeout(() => void propose(), 300);
    });
  }

  /** Another layout on the same land: a town seed of its own. */
  function proposeAgain() {
    unlessEdited(`Another layout: ${yourWork()}.`, 'Propose again', () => {
      app.code = { ...code, townSeed: roll() };
      void propose();
    });
  }

  /** Clear all was pressed: the hint for the first street shows until a street is drawn. */
  let cleared = $state(false);
  let clearRoads = 0;
  function clearEverything() {
    if (!editor.ready) return;
    const before = editor.draft;
    if (editor.apply(clearAll(before))) {
      cleared = true;
      clearRoads = editor.draft.roads.length;
    }
  }
  $effect(() => {
    if (cleared && plan && plan.roads.length > clearRoads) untrack(() => (cleared = false));
  });

  function rollName() {
    name = worldName(content, specOf(code.kind, optionsOf(code)).names, Math.random);
    app.townName = name;
  }

  // ---- Moving the town (a click on the land) ---------------------------------------------------

  /** "Move the town" is in hand: a click on the land puts the town's box there. */
  let moving = $state(false);
  /** The town's box where it would go, while the pointer is over the land (SVG path data, screen pixels). */
  let ghost = $state('');

  /** The main town's box (m): the one laid, else about a town's. */
  function boxOf(w: NeighbourhoodDraft | null): { w: number; d: number } {
    const a = w?.open?.areas[0];
    return a ? { w: a.w, d: a.d } : { w: 220, d: 180 };
  }

  /** The box's corner for the town centred on (`x`, `z`), kept on the land. */
  function siteAt(x: number, z: number): Pt {
    const box = boxOf(town);
    const W = app.land?.draft.width ?? LAND_SIZE[code.size];
    const D = app.land?.draft.depth ?? LAND_SIZE[code.size];
    const clamp = (v: number, hi: number) => Math.round(Math.max(0, Math.min(hi, v)));
    return [clamp(x - box.w / 2, W - box.w), clamp(z - box.d / 2, D - box.d)];
  }

  /** The box at `site`, on screen (at the land's height under the pointer `y`). */
  function boxGhost(site: Pt, y: number): string {
    const box = boxOf(town);
    const out = { x: 0, y: 0 };
    const corners: [number, number][] = [
      [site[0], site[1]],
      [site[0] + box.w, site[1]],
      [site[0] + box.w, site[1] + box.d],
      [site[0], site[1] + box.d],
    ];
    const points: string[] = [];
    for (const [x, z] of corners) {
      if (!menuScene.project(x, y, z, out)) return '';
      points.push(`${out.x.toFixed(1)} ${out.y.toFixed(1)}`);
    }
    return `M${points.join('L')}Z`;
  }

  const moveTool: StageTool = {
    down(x, y) {
      const p = menuScene.landAt(x, y);
      if (!p) return false;
      const site = siteAt(p.x, p.z);
      unlessEdited(`Moving the town lays out the streets again: ${yourWork()}.`, 'Move the town', () => {
        app.site = site;
        moving = false;
        ghost = '';
        void propose();
      });
      return true;
    },
    move: () => {},
    up: () => {},
    hover(x, y) {
      const p = menuScene.landAt(x, y);
      ghost = p ? boxGhost(siteAt(p.x, p.z), p.y) : '';
    },
    leave: () => (ghost = ''),
    cancel() {
      ghost = '';
      moving = false;
    },
  };
  const stageTool = $derived(making ? null : moving ? moveTool : editor.stageTool);

  function toggleMove() {
    moving = !moving;
    ghost = '';
    if (moving) {
      focus = null;
      hover = null;
      editor.take(null);
    }
  }
  // (An editor tool taken up puts "Move the town" down.)
  $effect(() => {
    if (editor.toolId) untrack(() => (moving = false));
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && moving) {
      moving = false;
      ghost = '';
    }
  }

  // ---- On and back -----------------------------------------------------------------------------

  /**
   * On to step 3 (Build) once the layout is settled and checked, unless a problem blocks it or the zones
   * would build fewer than two homes. A town built before from another layout is built again there:
   * asked first here if the player tweaked it.
   */
  async function next() {
    if (leaving) return;
    leaving = true;
    try {
      await editor.settled();
    } finally {
      leaving = false;
    }
    const l = app.layout;
    if (!l || editor.issues.blocking.length || fewHomes || app.screen !== 'streets') return;
    const w = app.world;
    if (w?.edited && w.key !== worldKeyOf(l, app.buildSeed)) {
      confirming = {
        text: `The layout changed since the town was built: it is built again, and ${changesWord(w.history?.edits ?? 0)} to it go.`,
        yes: 'Build it again',
        no: 'Keep editing',
        go: () => (app.screen = 'town'),
      };
      return;
    }
    app.screen = 'town';
  }

  function back() {
    clearTimeout(settling);
    job?.abort();
    app.screen = 'land';
  }

  onMount(async () => {
    if (!app.land) {
      // No land made yet: that is the first step.
      app.screen = 'land';
      return;
    }
    // The layout proposed before on this land, as it was left: shown again. Else (another land, or new choices) proposed now.
    if (app.layout && app.layout.key === townKey(app.land.key, code, app.site)) {
      const control = (job = new AbortController());
      making = 'The streets';
      done = new Set(['roads', 'homes', 'houses']);
      try {
        await menuScene.show(app.layout.draft, null, tick(control));
      } catch (err) {
        console.warn('[layout] showing the streets failed', err);
      } finally {
        if (job === control) {
          job = null;
          making = null;
        }
      }
      return;
    }
    await propose();
  });

  $effect(() => {
    menuScene.highlight({ hover: stageTool ? editor.hoverPlot : hover, selected: editor.selectedPlot ?? focus, marked: [], outlines: true });
  });
  $effect(() => () => menuScene.highlight({}));
</script>

{#snippet head()}
  <div class="name-field">
    <input aria-label="Town name" placeholder={town ? '' : 'Naming the town…'} bind:value={name} oninput={() => (app.townName = name.trim() || null)} maxlength="24" />
    <button class="btn ghost roll" aria-label="Another name" title="Another name" onclick={rollName}><Icon name="icon.ui.dice" size={18} /></button>
  </div>
  {#if town?.world}
    <p class="lede">{town.world.tagline}</p>
  {/if}
{/snippet}

{#snippet panel()}
  {#if confirming}
    <div class="asked"><SetupConfirm label="Your changes to the town" text={confirming.text} yes={confirming.yes} no={confirming.no} onanswer={answer} /></div>
  {/if}
  <section class="group" aria-labelledby="layout-head">
    <h2 class="eyebrow" id="layout-head">Your town's layout</h2>
    <p class="hint summary" data-summary>{zoned ? summaryLine(zoned) : ' '}</p>
    <div class="options">
      {#key unasked}
        {#each TOWN_FIELDS as f (f.key)}
          <div class="option">
            <span class="eyebrow sub">{f.label}</span>
            <Segmented fill label={f.label} value={code[f.key] as string} options={f.choices.map((c) => ({ ...c, disabled: !fits(f.key, c.value) }))} onchange={(v) => setOption(f.key, v)} />
          </div>
        {/each}
      {/key}
    </div>
    {#if ruledOut}
      <p class="hint">A {sizeLabel} land has no room for {ruledOut}: choose a larger one in step 1.</p>
    {/if}
    <div class="row">
      <button class="btn" disabled={!app.land} onclick={proposeAgain}><Icon name="icon.ui.dice" size={16} /> Propose again</button>
      <button class="btn" class:on={moving} aria-pressed={moving} disabled={!town || !!making} onclick={toggleMove}><Icon name="icon.ui.move" size={16} /> Move the town</button>
    </div>
    <div class="row">
      <button class="btn" disabled={!editor.ready || !!making} title="Every road, village and zone gone: bare land" onclick={clearEverything}><Icon name="icon.ui.trash" size={16} /> Clear all</button>
      <button class="btn" disabled={!editor.ready || !!making || !plan?.open?.uses} title="Every zone gone, the streets kept" onclick={() => editor.apply(clearZones(editor.draft))}><Icon name="icon.ui.eraser" size={16} /> Clear zones</button>
    </div>
    {#if moving}
      <p class="hint" role="status">Click the land where the town should go. <kbd>Esc</kbd> leaves it where it is.</p>
    {:else if cleared}
      <p class="hint" role="status">Bare land: take the Street tool and draw the first street in from the edge of the world, then paint zones beside it.</p>
    {/if}
  </section>

  <!-- The layout editor's slot: its tools (`LAYOUT_TOOLS`), the card of the tool in hand, undo and the Problems list. -->
  <div class="editor-slot" data-slot="town-editor">
    <TownEditor {editor} disabled={!town || !!making || moving} />
  </div>
{/snippet}

{#snippet stage()}
  {#if town}
    <TownStage
      {labels}
      {hover}
      selected={editor.selectedPlot ?? focus}
      {focus}
      shot="world"
      nearest={OVERVIEW_CLOSE}
      tool={stageTool}
      {ghost}
      onhover={(p) => (hover = p)}
      onselect={(p) => {
        focus = null;
        editor.selectShown(p);
      }}
    />
  {:else}
    <div></div>
  {/if}
{/snippet}

{#snippet figures()}
  {#if town && zoned}
    <span><b>{roadCount}</b> {roadCount === 1 ? 'road' : 'roads'}</span>
    <span><b>≈ {zoned.homes}</b> homes</span>
    <span><b>≈ {zoned.shops}</b> {zoned.shops === 1 ? 'shop' : 'shops'}</span>
    <span><b>{zoned.parks}</b> {zoned.parks === 1 ? 'park' : 'parks'}</span>
    <span><b>{zoned.public}</b> public</span>
    {#if villages}<span><b>{villages}</b> {villages === 1 ? 'village' : 'villages'}</span>{/if}
  {/if}
{/snippet}

{#snippet actions()}
  {#if blocked}
    <span class="blocked" role="status">A problem in the list keeps the streets from being built on</span>
  {:else if fewHomes && town && !making}
    <span class="blocked" role="status">Zone room for at least two homes along the streets</span>
  {/if}
  <button
    class="btn primary"
    disabled={!town || !!making || waiting || blocked || fewHomes || leaving}
    title={blocked ? 'Mend the problems in red first' : fewHomes ? 'Paint Homes beside a street' : undefined}
    onclick={next}>{leaving ? 'Bringing the streets up to date…' : STEPS[1].next}</button
  >
{/snippet}

<svelte:window onkeydown={onKey} />

<SetupShell label="The streets and zones" step={stepLabel(2)} back={{ label: 'Land', go: back }} {head} {panel} {stage} stats={figures} {actions} making={progress} />

<style>
  .asked {
    padding-top: 12px;
  }
  .summary {
    color: #3b4254;
    font-weight: 550;
    margin-bottom: 2px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }
  .row .btn {
    flex: 1 1 auto;
    justify-content: center;
    gap: 8px;
    height: 36px;
    font-size: 13px;
  }
  .row .btn.on {
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    box-shadow: 0 0 0 3px var(--accent-soft);
    color: var(--accent);
  }
  kbd {
    padding: 0 5px;
    border-radius: 4px;
    background: var(--hairline);
    font-family: inherit;
    font-size: 11px;
  }
  .editor-slot {
    display: contents;
  }
  .blocked {
    align-self: center;
    padding: 6px 12px;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--surface) 80%, transparent);
    color: #a3552c;
    font-size: 12.5px;
    font-weight: 600;
  }
</style>
