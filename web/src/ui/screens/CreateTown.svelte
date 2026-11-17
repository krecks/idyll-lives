<script lang="ts">
  /**
   * New game, step 3: Build. The town built from step 2's layout (the `build` job: an untouched layout
   * builds the proposal's own town), for the player to tweak with the editor's `BUILD_TOOLS` (today's
   * tools: Select, Street, Lot, Village, Remove, with Build up and Civic on the cards). "Build again"
   * builds it from the same layout with another build seed, asked first once the town has the
   * player's own changes (`app.world.edited`: "your N changes go").
   *
   * Going back to step 2 and on again without changing the layout keeps the tweaks (the built town's
   * key, `buildKey`, is the layout's and the build seed's). A street drawn here goes into the layout
   * too (`newGame.ts` `addedStreets`), so it doesn't make the town be built again.
   *
   * The editor (`townedit/`) works on the town's plan draft and shows its edits as ghosts until the
   * town is built again with them; the way on waits for that, and for its Problems list, and is
   * shut while a problem blocks it.
   */
  import { onMount, untrack } from 'svelte';
  import { TownHistory } from '../../game/edit/history';
  import { rename } from '../../game/edit/streets';
  import { menuScene } from '../../game/menuScene';
  import { NEIGHBOUR_CAP } from '../../game/town';
  import { keepHouses, Superseded, townJob } from '../../game/townJobs';
  import { specOf, worldName, type WorldStep } from '../../game/world';
  import { worldLine } from '../../game/worldOptions';
  import type { TownBuildStep } from '../../render/types';
  import { app } from '../app.svelte';
  import { toast } from '../state.svelte';
  import Icon from '../Icon.svelte';
  import TownStage, { type LotLabel } from '../kit/TownStage.svelte';
  import { services } from '../services';
  import { addedStreets, changesWord, layoutPlan, optionsOf, startCode, stepLabel, STEPS, worldKeyOf } from '../newGame';
  import SetupConfirm from './SetupConfirm.svelte';
  import SetupShell, { type Making } from './SetupShell.svelte';
  import { TownEditorState } from '../townedit/editor.svelte';
  import TownEditor from '../townedit/TownEditor.svelte';

  const { content } = services;
  /** A new seed: a different town every time. */
  const roll = () => ((Math.random() * 2 ** 31) >>> 0) || 1;

  const code = $derived(app.code ?? startCode('country', 0));
  /** The town shown (nobody living there yet): step 3's from before, else the one built now. */
  const town = $derived(app.world?.draft ?? null);
  /** The town's name as typed or rolled here (or in step 2), else the one it was laid with. */
  let name = $state(app.townName ?? app.world?.draft.name ?? app.layout?.draft.name ?? '');
  let hover = $state<number | null>(null);
  let focus = $state<number | null>(null);

  /** The town editor (its tools, cards, undo and Problems list), over the town shown. */
  const editor = new TownEditorState({
    content,
    step: 'build',
    getName: () => name,
    setName(typed) {
      name = typed;
      app.townName = typed.trim() || null;
    },
    glide: (plot) => (focus = plot),
    // A street drawn on the built town goes into the layout as well, and the built town is keyed by the layout as it
    // is now: going back to step 2 and on again keeps the town as tweaked.
    edited(before, after) {
      const l = app.layout;
      const w = app.world;
      if (!l || !w) return;
      const was = layoutPlan(l);
      const now = addedStreets(was, before, after);
      if (now === was) return;
      const from = l.plan ?? l.proposal.plan;
      const history = l.history ?? new TownHistory(from);
      history.push(now);
      app.layout = { ...l, plan: from, history, edited: true };
      app.world = { ...w, key: worldKeyOf(app.layout, app.buildSeed) };
    },
  });
  // (The editor's own writes to `app.world` come back here: it keeps its session over them.)
  $effect(() => {
    const w = app.world;
    untrack(() => editor.attach(making ? null : w));
  });
  $effect(() => () => editor.dispose());
  /** The town as the edits left it (the figures follow the edits at once; the town shown follows once they pause). */
  const plan = $derived(editor.ready ? editor.draft : town);
  /** On to step 4 asked for: waiting for the town to be brought up to date with the edits, and checked. */
  let leaving = $state(false);
  const blocked = $derived(editor.issues.blocking.length > 0);

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
    if (!w) return [];
    const shown = w.slots.filter((s) => s.kind !== 'house' || s.lot === 'farm');
    const times = new Map<string, number>();
    for (const s of shown) times.set(s.name, (times.get(s.name) ?? 0) + 1);
    return shown.map((s) => {
      const area = s.area ?? 0;
      return { plot: s.index, title: s.name, sub: area > 0 && (times.get(s.name) ?? 0) > 1 ? w.open?.areas[area]?.name : undefined, kind: s.kind === 'house' ? 'sale' : 'park' };
    });
  });

  /**
   * Building the town, as the progress panel lists it: in the town worker (`WorldStep`, in turn), then
   * built in 3D (`TownBuildStep`, side by side: they tick off in any order).
   */
  const MAKING: readonly { key: WorldStep | TownBuildStep; label: string }[] = [
    { key: 'roads', label: 'Laying the roads' },
    { key: 'homes', label: 'Placing the homes' },
    { key: 'houses', label: 'Building the houses' },
    { key: 'hills', label: 'Settling the land' },
    { key: 'woods', label: 'Planting the woods' },
    { key: 'town', label: 'Building the town' },
    { key: 'grass', label: 'Sowing the grass' },
  ];

  /** The town being built (in the town worker, then shown); a newer one replaces it. */
  let job: AbortController | null = null;
  /** The progress panel's title while a town is built or shown (null: none is). */
  let making = $state<string | null>(null);
  let done = $state.raw<ReadonlySet<string>>(new Set());
  const progress = $derived<Making | null>(making ? { title: making, steps: MAKING, done } : null);
  const tick = (control: AbortController) => (step: string) => {
    if (job === control) done = new Set([...done, step]);
  };

  /** The town built from the layout as it is now with the build seed, and shown; the one being built meanwhile is dropped. */
  async function build() {
    const layout = app.layout;
    if (!layout) return;
    job?.abort();
    const control = (job = new AbortController());
    // (The editor lets go of the town being replaced: no edit of it is settled over the new one.)
    editor.attach(null);
    const seed = app.buildSeed;
    const key = worldKeyOf(layout, seed);
    making = app.world ? 'Building it again' : 'Building the town';
    done = new Set();
    const step = tick(control);
    try {
      const made = await townJob(content, { kind: 'build', plan: layoutPlan(layout), proposal: layout.proposal, seed }, control.signal, step);
      if (job !== control) return;
      // Its houses and preview come with it: the menu draws it without making them again.
      keepHouses(made);
      menuScene.prepare(made.draft, made.preview);
      app.world = { draft: made.draft, key, edited: false, ...(made.plan ? { plan: made.plan } : {}), ...(made.marks ? { marks: made.marks } : {}) };
      // (Another town: its households are made again in step 4.)
      app.townOf = '';
      name = app.townName ?? made.draft.name;
      focus = null;
      hover = null;
      // (The panel stays until it is built and on screen.)
      await menuScene.show(made.draft, null, step);
    } catch (err) {
      if (err instanceof Superseded) return;
      console.error('[build] building the town failed:', err);
      toast('The town could not be built from that layout. Change it in step 2, or build again.');
    } finally {
      if (job === control) {
        job = null;
        making = null;
        editor.attach(app.world);
      }
    }
  }
  $effect(() => () => job?.abort());

  let confirming = $state<{ text: string; yes: string; go: () => void } | null>(null);
  function answer(yes: boolean) {
    const c = confirming;
    confirming = null;
    if (yes) c?.go();
  }

  /** Build again: another build seed on the same layout, asked first if the town has the player's own changes. */
  function buildAgain() {
    const go = () => {
      app.buildSeed = roll();
      void build();
    };
    const w = app.world;
    if (w?.edited) confirming = { text: `Building the town again from the same layout: ${changesWord(editor.edits)} to it go.`, yes: 'Build again', go };
    else go();
  }

  function rollName() {
    name = worldName(content, specOf(code.kind, optionsOf(code)).names, Math.random);
    app.townName = name;
  }

  // ---- On and back -----------------------------------------------------------------------------

  /**
   * On to step 4 with the town as named here (its neighbours are made there, or kept if it is the
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
    job?.abort();
    app.screen = 'streets';
  }

  onMount(async () => {
    const layout = app.layout;
    if (!app.land || !layout) {
      // No layout yet: that is step 2 (or step 1, with no land either).
      app.screen = app.land ? 'streets' : 'land';
      return;
    }
    // The town built before from this layout with this seed, as it was left: shown again. Else built now.
    if (app.world && app.world.key === worldKeyOf(layout, app.buildSeed)) {
      const control = (job = new AbortController());
      making = 'The town';
      done = new Set(['roads', 'homes', 'houses']);
      try {
        await menuScene.show(app.world.draft, null, tick(control));
      } catch (err) {
        console.warn('[build] showing the town failed', err);
      } finally {
        if (job === control) {
          job = null;
          making = null;
        }
      }
      return;
    }
    await build();
  });

  $effect(() => {
    menuScene.highlight({ hover: editor.stageTool ? editor.hoverPlot : hover, selected: editor.selectedPlot ?? focus, marked: [], outlines: true });
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
    <p class="hint summary">{summary || ' '}</p>
    <div class="row">
      <button class="btn" disabled={!app.layout || !!making} title="The same layout built with other houses and places" onclick={buildAgain}><Icon name="icon.ui.dice" size={16} /> Build again</button>
    </div>
    <p class="hint">Built from your streets and zones. Change the layout in step 2; tweak the town here.</p>
  </section>

  <!--
    The town editor's slot: its tool palette (`townedit/tools.ts` `BUILD_TOOLS`, one in hand at a
    time through the stage's `tool`), the card of the tool in hand, rename, undo and the Problems list.
    An edit marks the town as the player's own (`app.world.edited`): what builds it again then asks first.
  -->
  <div class="editor-slot" data-slot="town-editor">
    <TownEditor {editor} disabled={!town || !!making} />
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
      tool={making ? null : editor.stageTool}
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
  <button class="btn primary" disabled={!town || !!making || blocked || leaving} title={blocked ? 'Mend the problems in red first' : undefined} onclick={next}>{leaving ? 'Bringing the town up to date…' : STEPS[2].next}</button>
{/snippet}

<SetupShell label="The town" step={stepLabel(3)} back={{ label: 'Streets & zones', go: back }} {head} {panel} {stage} stats={figures} {actions} making={progress} />

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
