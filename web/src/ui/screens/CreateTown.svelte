<script lang="ts">
  /**
   * New game, step 2: the town, laid on step 1's land (the town job's `town`). Its name, how big it
   * is and how many villages lie round it (choices the land is too small for are ruled out), another
   * town from a new seed, and the town moved by a click on the land. The editor's tools (UE) come
   * into the panel's slot for them; what lays the town out again asks first once it has the
   * player's own changes (`app.world.edited`).
   *
   * The editor (`townedit/`) works on the town's plan draft and shows its edits as ghosts until the
   * town is built again with them; the way on waits for that, and for its Problems list, and is
   * shut while a problem blocks it.
   */
  import { onMount, untrack } from 'svelte';
  import { rename } from '../../game/edit/streets';
  import { menuScene } from '../../game/menuScene';
  import type { Pt } from '../../game/roads';
  import { NEIGHBOUR_CAP, type NeighbourhoodDraft } from '../../game/town';
  import { keepHouses, Superseded, townJob } from '../../game/townJobs';
  import { LAND_SIZE, specOf, townFits, worldName, type WorldStep } from '../../game/world';
  import { OPTION_FIELDS, worldLine } from '../../game/worldOptions';
  import type { TownBuildStep } from '../../render/types';
  import { app } from '../app.svelte';
  import { toast } from '../state.svelte';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import TownStage, { type LotLabel, type StageTool } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import { fitted, optionsOf, SIZE_CHOICES, startCode, townKey, type Chosen } from '../newGame';
  import SetupConfirm from './SetupConfirm.svelte';
  import SetupShell, { type Making } from './SetupShell.svelte';
  import { TownEditorState } from '../townedit/editor.svelte';
  import TownEditor from '../townedit/TownEditor.svelte';

  const { content } = services;
  /** A new seed: a different town every time. */
  const roll = () => (Math.random() * 2 ** 31) >>> 0;

  const code = $derived(app.code ?? startCode('country', 0));
  /** The town shown (nobody living there yet): step 2's from before, else the one laid now. */
  const town = $derived(app.world?.draft ?? null);
  /** The town's name as typed or rolled here, else the one it was laid with. */
  let name = $state(app.townName ?? app.world?.draft.name ?? '');
  let hover = $state<number | null>(null);
  let focus = $state<number | null>(null);

  /** The town editor (its tools, cards, undo and Problems list), over the town shown. */
  const editor = new TownEditorState({
    content,
    getName: () => name,
    setName(typed) {
      name = typed;
      app.townName = typed.trim() || null;
    },
    glide: (plot) => (focus = plot),
  });
  // (The editor's own writes to `app.world` come back here: it keeps its session over them.)
  $effect(() => {
    const w = app.world;
    untrack(() => editor.attach(making ? null : w));
  });
  $effect(() => () => editor.dispose());
  /** The town as the edits left it (the figures follow the edits at once; the town shown follows once they pause). */
  const plan = $derived(editor.ready ? editor.draft : town);
  /** On to step 3 asked for: waiting for the town to be brought up to date with the edits, and checked. */
  let leaving = $state(false);
  const blocked = $derived(editor.issues.blocking.length > 0);

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

  const homes = $derived(plan ? plan.slots.filter((s) => s.kind === 'house').length : 0);
  const farms = $derived(plan ? plan.slots.filter((s) => s.lot === 'farm').length : 0);
  const places = $derived(plan ? plan.slots.filter((s) => s.kind !== 'house').length : 0);
  const roadCount = $derived(plan ? plan.roads.length : 0);
  const villages = $derived(plan?.open ? Math.max(0, plan.open.areas.length - 1) : 0);
  /** The town in a line: homes, its size, land, how many residents it holds. */
  const summary = $derived(plan ? worldLine({ homes, villages, width: plan.width, depth: plan.depth, town: code.town, cap: NEIGHBOUR_CAP }) : '');

  // Public places and farms have their names on them; plain houses are left untagged (the town reads better).
  // A big world's areas each pick their own shops, so one name can come up in two of them: a repeat
  // says which village it is in (the town's own needs no saying).
  const labels = $derived.by((): LotLabel[] => {
    const w = town;
    if (!w || moving) return [];
    const shown = w.slots.filter((s) => s.kind !== 'house' || s.lot === 'farm');
    const times = new Map<string, number>();
    for (const s of shown) times.set(s.name, (times.get(s.name) ?? 0) + 1);
    return shown.map((s) => {
      const area = s.area ?? 0;
      return { plot: s.index, title: s.name, sub: area > 0 && (times.get(s.name) ?? 0) > 1 ? w.open?.areas[area]?.name : undefined, kind: s.kind === 'house' ? 'sale' : 'park' };
    });
  });

  /**
   * Laying out the town, as the progress panel lists it: laid out in the town worker (`WorldStep`, in
   * turn), then built in 3D (`TownBuildStep`, side by side: they tick off in any order).
   */
  const STEPS: readonly { key: WorldStep | TownBuildStep; label: string }[] = [
    { key: 'roads', label: 'Laying the roads' },
    { key: 'homes', label: 'Placing the homes' },
    { key: 'houses', label: 'Building the houses' },
    { key: 'hills', label: 'Settling the land' },
    { key: 'woods', label: 'Planting the woods' },
    { key: 'town', label: 'Building the town' },
    { key: 'grass', label: 'Sowing the grass' },
  ];

  /** The town being laid (in the town worker, then built); a newer one replaces it. */
  let job: AbortController | null = null;
  /** The progress panel's title while a town is laid or built (null: none is). */
  let making = $state<string | null>(null);
  /** An option changed: the town is laid again once they settle; meanwhile it can't be taken on. */
  let settling: ReturnType<typeof setTimeout> | undefined;
  let waiting = $state(false);
  let done = $state.raw<ReadonlySet<string>>(new Set());
  const progress = $derived<Making | null>(making ? { title: making, steps: STEPS, done } : null);
  const tick = (control: AbortController) => (step: string) => {
    if (job === control) done = new Set([...done, step]);
  };

  /** The town for the land and the code as they are now, laid and built; the one being laid meanwhile is dropped. */
  async function lay() {
    clearTimeout(settling);
    waiting = false;
    const land = app.land;
    if (!land) return;
    job?.abort();
    const control = (job = new AbortController());
    // (The editor lets go of the town being replaced: no edit of it is settled over the new one.)
    editor.attach(null);
    const asked = code;
    const site = app.site;
    const key = townKey(land.key, asked, site);
    making = app.world ? 'Another town' : 'Laying out the town';
    done = new Set();
    const step = tick(control);
    try {
      const made = await townJob(
        content,
        { kind: 'town', land: land.draft, seed: asked.townSeed ?? asked.seed, name: app.townName ?? undefined, options: optionsOf(asked), ...(site ? { site } : {}), waterKept: land.waterKept },
        control.signal,
        step,
      );
      if (job !== control) return;
      // Its houses and preview come with it: the menu draws it without making them again.
      keepHouses(made);
      menuScene.prepare(made.draft, made.preview);
      app.world = { draft: made.draft, key, edited: false, ...(made.plan ? { plan: made.plan } : {}) };
      name = app.townName ?? made.draft.name;
      focus = null;
      hover = null;
      // (The panel stays until it is built and on screen.)
      await menuScene.show(made.draft, null, step);
    } catch (err) {
      if (err instanceof Superseded) return;
      console.error('[town] laying out the town failed:', err);
      // (On a shaped land the generator says why it couldn't: no dry land big enough.)
      toast(err instanceof Error && err.message && app.land?.draft.open?.land?.edited ? err.message : 'That town could not be laid out. Try another.');
    } finally {
      if (job === control) {
        job = null;
        making = null;
        editor.attach(app.world);
      }
    }
  }
  $effect(() => () => {
    clearTimeout(settling);
    job?.abort();
  });

  /** Asked first if the town has the player's own changes (laying it out again throws them away). */
  function unlessEdited(text: string, yes: string, go: () => void) {
    if (app.world?.edited) confirming = { text, yes, go };
    else go();
  }
  let confirming = $state<{ text: string; yes: string; go: () => void } | null>(null);
  /** Bumped when a choice is not made after all (the switches show the choice as it was). */
  let unasked = $state(0);
  function answer(yes: boolean) {
    const c = confirming;
    confirming = null;
    if (yes) c?.go();
    else unasked++;
  }

  /** The town or the villages changed: the town laid again with them, on the same seed (once the options have stood for 300 ms). */
  function setOption(key: 'town' | 'villages', value: string) {
    if (code[key] === value) return;
    const label = key === 'town' ? 'town' : 'villages';
    unlessEdited(`Changing the ${label} lays the town out again: your changes to it go.`, `Change the ${label}`, () => {
      app.code = fitted({ ...code, [key]: value } as Chosen);
      clearTimeout(settling);
      waiting = true;
      settling = setTimeout(() => void lay(), 300);
    });
  }

  /** Another town on the same land: a seed of its own. */
  function another() {
    unlessEdited('Another town leaves your changes to this one behind.', 'Another town', () => {
      app.code = { ...code, townSeed: roll() };
      void lay();
    });
  }

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
      unlessEdited('Moving the town lays it out again: your changes to it go.', 'Move the town', () => {
        app.site = site;
        moving = false;
        ghost = '';
        void lay();
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
   * On to step 3 with the town as named here (its neighbours are made there, or kept if it is the
   * town they live in): once the town is up to date with the edits and checked, unless a problem blocks it.
   */
  async function next() {
    if (leaving) return;
    leaving = true;
    try {
      await editor.settled();
    } finally {
      leaving = false;
    }
    const w = app.world;
    if (!w || editor.issues.blocking.length || app.screen !== 'town') return;
    const named = name.trim();
    if (named && named !== w.draft.name) {
      // (The main town is its first area: it is named with it.)
      const renamed = rename(w.draft, { kind: 'town' }, named);
      app.world = { ...w, draft: 'draft' in renamed ? renamed.draft : { ...w.draft, name: named } };
    }
    app.screen = 'household';
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
    // The town laid before on this land, as it was: shown again. Else (another land, or new choices) laid now.
    if (app.world && app.world.key === townKey(app.land.key, code, app.site)) {
      const control = (job = new AbortController());
      making = 'The town';
      done = new Set(['roads', 'homes', 'houses']);
      try {
        await menuScene.show(app.world.draft, null, tick(control));
      } catch (err) {
        console.warn('[town] showing the town failed', err);
      } finally {
        if (job === control) {
          job = null;
          making = null;
        }
      }
      return;
    }
    await lay();
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
    <div class="asked"><SetupConfirm label="Your changes to the town" text={confirming.text} yes={confirming.yes} no="Keep my town" onanswer={answer} /></div>
  {/if}
  <section class="group" aria-labelledby="town-head">
    <h2 class="eyebrow" id="town-head">Your town</h2>
    <p class="hint summary">{summary || ' '}</p>
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
      <button class="btn" disabled={!app.land} onclick={another}><Icon name="icon.ui.dice" size={16} /> Another town</button>
      <button class="btn" class:on={moving} aria-pressed={moving} disabled={!town || !!making} onclick={toggleMove}><Icon name="icon.ui.move" size={16} /> Move the town</button>
    </div>
    {#if moving}
      <p class="hint" role="status">Click the land where the town should go. <kbd>Esc</kbd> leaves it where it is.</p>
    {/if}
  </section>

  <!--
    The town editor's slot (UE): its tool palette (`townedit/tools.ts` `TOWN_TOOLS`, one in hand at a
    time through the stage's `tool`), the card of the tool in hand, rename, undo and the Problems list.
    An edit marks the town as the player's own (`app.world.edited`): what lays it out again then asks first.
  -->
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
  {#if town}
    <span><b>{roadCount}</b> {roadCount === 1 ? 'road' : 'roads'}</span>
    <span><b>{homes}</b> homes</span>
    {#if farms}<span><b>{farms}</b> {farms === 1 ? 'farm' : 'farms'}</span>{/if}
    <span><b>{places}</b> public {places === 1 ? 'place' : 'places'}</span>
    {#if villages}<span><b>{villages}</b> {villages === 1 ? 'village' : 'villages'}</span>{/if}
  {/if}
{/snippet}

{#snippet actions()}
  {#if blocked}
    <span class="blocked" role="status">A problem in the list keeps the town from its households</span>
  {/if}
  <button class="btn primary" disabled={!town || !!making || waiting || blocked || leaving} title={blocked ? 'Mend the problems in red first' : undefined} onclick={next}>{leaving ? 'Bringing the town up to date…' : 'Choose a household →'}</button>
{/snippet}

<svelte:window onkeydown={onKey} />

<SetupShell label="The town" step="Step 2 of 4 · Town" back={{ label: 'Land', go: back }} {head} {panel} {stage} stats={figures} {actions} making={progress} />

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
