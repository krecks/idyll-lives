/**
 * The town editor as the screens see it, in step 2 (Streets & zones: the layout) and step 3 (Build:
 * the built town): the edit session (`session.ts`: the plan draft, the settled town, undo, the
 * Problems list) with what the tools and cards share besides: the tool in hand, what is picked and
 * what is under the pointer (kept by what they are, `pick.ts`), the tools' own ghosts, step 2's zones
 * and a line of hint, all drawn over the town through `menuScene`.
 *
 * `CreateLayout.svelte` and `CreateTown.svelte` each make one for their step and give it what they
 * show (`attach`); `TownEditor.svelte` is its panel. What the edits make is kept in `app.layout` (step
 * 2) or `app.world` (step 3) (`plan`, `history`, `edited`; `draft` is the settled town the next steps
 * read), so going back and forth through the steps keeps them.
 */

import type { Content } from '../../content/content';
import type { Edited } from '../../game/edit/edited';
import { lotKey, slotOf } from '../../game/edit/lots';
import { menuScene } from '../../game/menuScene';
import { holdTownWorker } from '../../game/townJobs';
import { loadTemplates, type NeighbourhoodDraft, type Templates } from '../../game/town';
import type { LandCursor, TownGhosts } from '../../render/types';
import type { LaidTown, Layout } from '../app.svelte';
import { app } from '../app.svelte';
import { toast } from '../state.svelte';
import { checkTown } from './checkJob';
import { buildingBox, editGhosts, joinGhosts, lotBox, roadGhost } from './ghosts';
import { hatchedUses } from './layoutApi';
import { find, keyOf, pick, type Picked, type PickWhat } from './pick';
import type { Issue, TownIssues } from './problems';
import { EditSession, type SettleTiming } from './session';
import { settle, type Settled } from './settle';
import { toolsFor, type EditStep, type TownEditor, type TownStageTool, type TownTool, type TownToolId } from './tools';
import { sentence } from './words';

/** What the screen gives the editor: its step, the town's name as the step's field has it, and the camera's glide to a lot of the town on screen. */
export interface EditorHooks {
  content: Content;
  /** Step 2's layout (`app.layout`, its tools `LAYOUT_TOOLS`) or step 3's built town (`app.world`, `BUILD_TOOLS`). Step 3 if not said. */
  step?: EditStep;
  /** An edit was taken on: the plan before it and after (step 3's streets go into the layout too). */
  edited?(before: NeighbourhoodDraft, after: NeighbourhoodDraft): void;
  /** The town's name as typed (step 2's own field), and a new one typed in the editor. */
  getName(): string;
  setName(name: string): void;
  /** The camera to a lot of the town on screen (its plot), or back to the whole world (null). */
  glide(plot: number | null): void;
}

export class TownEditorState implements TownEditor {
  /** Bumped whenever anything the panel shows changes (the session is no Svelte state). */
  version = $state(0);
  /** The house templates (the tools can't run before they are loaded). */
  templates = $state.raw<Templates | null>(null);
  /** The tool in hand (its `TownTool.id`), or none: the stage as the step has it, a click on a lot picks it. */
  toolId = $state<TownToolId | null>(null);
  /** The step edited, and its tools in palette order. */
  readonly step: EditStep;
  readonly tools: readonly TownTool[];
  /** What is picked (`keyOf`), and what is under the pointer (on the plan). */
  private pickedKey = $state<string | null>(null);
  hovered = $state.raw<Picked | null>(null);
  /** A line about what the tool in hand would do where the pointer is (why it can't, say), and the last edit refused. */
  hintText = $state<string | null>(null);
  note = $state<string | null>(null);
  private session: EditSession | null = null;
  /** Lets go of the town worker held while a town is edited. */
  private letWorkerGo: (() => void) | null = null;
  private toolGhosts: Partial<TownGhosts> | null = null;
  private readonly stages = new Map<string, TownStageTool>();
  private seeds = (Math.random() * 2 ** 31) >>> 0;
  private noteTimer: ReturnType<typeof setTimeout> | undefined;
  /** The edit ghosts as last worked out, and the drafts they were of. */
  /** What was drawn last: the edits' ghosts and the rest's (as JSON). */
  private drawn: { edits: TownGhosts | null; key: string } | null = null;
  private editGhostsOf: { settled: NeighbourhoodDraft; plan: NeighbourhoodDraft; ghosts: TownGhosts | null } | null = null;
  /** Step 2's zones shown: a stroke's plan over the plan's own (`showUses`), and the plan whose zones are drawn. */
  private strokeUses: NeighbourhoodDraft | null = null;
  private usesDrawn: NeighbourhoodDraft | null | undefined = undefined;

  constructor(private readonly hooks: EditorHooks) {
    this.step = hooks.step ?? 'build';
    this.tools = toolsFor(this.step);
    // (The bench drives the editor and reads its settles' timings: `tools/bench/create.mjs`.)
    if (import.meta.env.DEV) Object.assign(globalThis, { __townEditor: this });
    void loadTemplates().then((t) => {
      this.templates = t;
      this.changed();
    });
  }

  get content(): Content {
    return this.hooks.content;
  }

  /** What the step edits as `app` keeps it: step 2's layout, step 3's built town. */
  private get target(): LaidTown | Layout | null {
    return this.step === 'layout' ? app.layout : app.world;
  }
  private set target(v: LaidTown | Layout) {
    if (this.step === 'layout') app.layout = v as Layout;
    else app.world = v as LaidTown;
  }

  /** The plan draft as the edits left it (the town as laid, before any). */
  get draft(): NeighbourhoodDraft {
    void this.version;
    return this.session?.plan ?? this.target?.draft ?? EMPTY;
  }

  /** The house templates; only asked for once they are loaded (`ready`). */
  get t(): Templates {
    if (!this.templates) throw new Error('The house templates are not loaded yet.');
    return this.templates;
  }

  /** The editor can take edits: a town to edit and the templates loaded. */
  get ready(): boolean {
    void this.version;
    return !!this.session && !!this.templates;
  }

  get canUndo(): boolean {
    void this.version;
    return !!this.session?.canUndo;
  }
  get canRedo(): boolean {
    void this.version;
    return !!this.session?.canRedo;
  }
  get edits(): number {
    void this.version;
    return this.session?.edits ?? 0;
  }
  /** A settle under way, or edits waiting for one. */
  get settling(): boolean {
    void this.version;
    return !!this.session && (this.session.settling || this.session.pending);
  }
  get issues(): TownIssues {
    void this.version;
    return this.session?.issues ?? { blocking: [], warnings: [] };
  }
  /** The Problems list is of the plan as it stands. */
  get checked(): boolean {
    void this.version;
    return this.session?.checked ?? true;
  }
  /** How long the settles took (the bench reads them). */
  get timings(): readonly SettleTiming[] {
    return this.session?.timings ?? [];
  }

  /** The tool in hand. */
  get tool(): TownTool | null {
    return this.tools.find((t) => t.id === this.toolId) ?? null;
  }

  /** What the town stage does with the pointer: the tool in hand's (one made per tool, kept: the stage keys on it). */
  get stageTool(): TownStageTool | null {
    const tool = this.tool;
    if (!tool || !this.ready) return null;
    let stage = this.stages.get(tool.id);
    if (!stage) this.stages.set(tool.id, (stage = tool.stage(this)));
    return stage;
  }

  /** A tool taken in hand (null: put down; the same one again puts it down). */
  take(id: TownToolId | null): void {
    // (What the tool put down was drawing is dropped with it; a tool this step hasn't is none.)
    const was = this.toolId;
    this.toolId = id === this.toolId || !this.tools.some((t) => t.id === id) ? null : id;
    if (was && was !== this.toolId) this.stages.get(was)?.cancel();
    this.preview(null);
    this.hint(null);
    this.hover(null);
    this.brush(null);
  }

  // ---- the town given ------------------------------------------------------------------------

  /**
   * The town the step shows from now on (`app.layout` or `app.world`): the session over it kept if it
   * is the town already being edited (the editor's own writes to `app` come back here), else a new
   * one, with the edits made before if the town has them. Null: none (the editor lets go).
   */
  attach(world: LaidTown | Layout | null): void {
    const s = this.session;
    if (s && world && (world.history ? world.history === s.history : s.history.base === (world.plan ?? world.draft) && s.edits === 0)) return;
    s?.dispose();
    this.letWorkerGo?.();
    this.letWorkerGo = null;
    this.session = null;
    this.editGhostsOf = null;
    this.pickedKey = null;
    this.hovered = null;
    this.toolGhosts = null;
    this.strokeUses = null;
    this.note = null;
    if (world) {
      // (The town worker kept while the town is edited: a fresh one would make every house again.)
      this.letWorkerGo = holdTownWorker();
      const session: EditSession = (this.session = new EditSession(
        world.draft,
        {
          settle,
          show: (settled, plan) => this.show(session, settled, plan),
          changed: () => this.changed(),
          check: (plan, base) => checkTown(this.hooks.content, plan, base),
          report: (why) => toast(sentence(why)),
        },
        world.history,
        world.plan,
        world.marks,
      ));
    }
    this.changed();
  }

  /** The editor let go (the step left): its ghosts, zones, brush and lights taken off the town. */
  dispose(): void {
    this.attach(null);
    clearTimeout(this.noteTimer);
    menuScene.ghosts(null);
    menuScene.brush(null);
    menuScene.setUses(null);
    this.usesDrawn = undefined;
  }

  /** The settled town put on screen, and kept as the town the next steps read (`draft`). */
  private async show(session: EditSession, settled: Settled, plan: NeighbourhoodDraft): Promise<void> {
    const { draft, preview, marks } = settled;
    menuScene.prepare(draft, preview);
    await menuScene.show(draft, null);
    const w = this.target;
    // (Not if the town was laid again meanwhile: this session's edits are of the one before.)
    if (!w || this.session !== session) return;
    const { marks: _was, ...rest } = w;
    this.target = { ...rest, draft, plan, history: session.history, edited: session.edits > 0, ...(marks ? { marks } : {}) };
    // (The built town changed: its households are made again in step 4.)
    if (this.step === 'build') app.townOf = '';
  }

  /** Something changed: the panel told, the ghosts drawn again, `app` told of the edits. */
  private changed(): void {
    this.version++;
    const s = this.session;
    const w = this.target;
    if (s && w && (w.plan !== s.plan || w.edited !== s.edits > 0) && (s.edits > 0 || w.history)) {
      this.target = { ...w, plan: s.plan, history: s.history, edited: s.edits > 0 };
    }
    // What was picked may have gone with the edit.
    if (this.pickedKey && s && !find(s.plan, this.pickedKey)) this.pickedKey = null;
    this.redraw();
  }

  // ---- edits -----------------------------------------------------------------------------------

  apply(edited: Edited, run = false): boolean {
    const s = this.session;
    if (!s) return false;
    const before = s.plan;
    const why = s.apply(edited, run);
    if (why) {
      this.refuse(why);
      return false;
    }
    this.note = null;
    if (s.plan !== before) this.hooks.edited?.(before, s.plan);
    return true;
  }

  refuse(why: string): void {
    this.note = sentence(why);
    toast(this.note);
    clearTimeout(this.noteTimer);
    this.noteTimer = setTimeout(() => (this.note = null), 6000);
  }

  undo(): void {
    this.session?.undo();
  }
  redo(): void {
    this.session?.redo();
  }
  reset(): void {
    this.session?.reset();
  }

  /** The town settled and on screen, and its problems checked, as the plan stands (the way on waits for it). */
  async settled(): Promise<void> {
    await this.session?.ready();
  }

  seed(): number {
    this.seeds = (Math.imul(this.seeds ^ (this.seeds >>> 15), 0x2c1b3c6d) + 0x9e3779b9) >>> 0;
    return this.seeds;
  }

  // ---- what is picked and pointed at -------------------------------------------------------------

  get selected(): Picked | null {
    void this.version;
    const key = this.pickedKey;
    const s = this.session;
    return key && s ? find(s.plan, key) : null;
  }

  select(p: Picked | null): void {
    this.pickedKey = p && this.session ? keyOf(this.session.plan, p) : null;
    this.redraw();
  }

  hover(p: Picked | null): void {
    const same = JSON.stringify(p) === JSON.stringify(this.hovered);
    if (same) return;
    this.hovered = p;
    this.redraw();
  }

  /** A lot of the town on screen (a plot of `app.world.draft`) picked: the same lot of the plan. */
  selectShown(plot: number | null): void {
    const s = this.session;
    const lot = plot === null || !s ? null : s.settled.slots[plot];
    const slot = lot ? slotOf(s!.plan, lotKey(lot)) : -1;
    this.select(slot >= 0 ? { kind: 'lot', slot } : null);
  }

  /** The plot of the town on screen that lot `slot` of the plan is (null: it isn't built yet). */
  shownPlot(slot: number | null | undefined): number | null {
    const s = this.session;
    const lot = slot === null || slot === undefined || !s ? undefined : s.plan.slots[slot];
    if (!lot || !s) return null;
    const at = slotOf(s.settled, lotKey(lot));
    return at >= 0 ? at : null;
  }

  /** The lot picked and the one pointed at, as plots of the town on screen (for its lights: `menuScene.highlight`). */
  get selectedPlot(): number | null {
    const p = this.selected;
    return p?.kind === 'lot' ? this.shownPlot(p.slot) : null;
  }
  get hoverPlot(): number | null {
    void this.version;
    const p = this.hovered;
    return p?.kind === 'lot' ? this.shownPlot(p.slot) : null;
  }

  /** The point on the ground under a screen point (CSS pixels), if the town is there. */
  ground(x: number, y: number): { x: number; z: number } | null {
    const at = menuScene.landAt(x, y);
    return at && { x: at.x, z: at.z };
  }

  pick(x: number, y: number, what?: PickWhat): Picked | null {
    const g = this.ground(x, y);
    return g && this.session && this.templates ? pick(this.session.plan, this.templates, g.x, g.z, what) : null;
  }

  preview(g: Partial<TownGhosts> | null): void {
    this.toolGhosts = g;
    this.redraw();
  }

  hint(text: string | null): void {
    if (text !== this.hintText) this.hintText = text;
  }

  brush(cursor: LandCursor | null): void {
    menuScene.brush(cursor);
  }

  showUses(plan: NeighbourhoodDraft | null): void {
    this.strokeUses = plan;
    this.drawUses();
  }

  /** Step 2's zones drawn over the town, as a stroke under way or the plan has them, hatched where nothing would be built (when they changed). */
  private drawUses(): void {
    const plan = this.step === 'layout' && this.session ? (this.strokeUses ?? this.session.plan) : null;
    if (plan === this.usesDrawn) return;
    const was = this.usesDrawn;
    this.usesDrawn = plan;
    // (An edit that left the zones and the roads as they were leaves the overlay as it is.)
    if (plan && was && plan.open?.uses === was.open?.uses && plan.roads === was.roads) return;
    const layer = plan?.open?.uses ?? null;
    menuScene.setUses(layer, layer && plan ? hatchedUses(plan) : null);
  }

  /** The camera to where a problem is (the lot of the town on screen nearest it). */
  glide(issue: Issue): void {
    const s = this.session;
    const t = this.templates;
    if (!s || !t || !issue.at) return;
    const { x, z } = issue.at;
    let best: number | null = null;
    let near = Infinity;
    for (const lot of s.settled.slots) {
      const b = lotBox(t, lot);
      const d = Math.hypot(b.x + b.w / 2 - x, b.z + b.d / 2 - z);
      if (d < near) [best, near] = [lot.index, d];
    }
    if (issue.lot !== undefined) {
      const shown = this.shownPlot(issue.lot);
      if (shown !== null) best = shown;
      this.select({ kind: 'lot', slot: issue.lot });
    }
    this.hooks.glide(best);
  }

  get townName(): string {
    return this.hooks.getName();
  }
  set townName(name: string) {
    this.hooks.setName(name);
  }

  // ---- drawing ---------------------------------------------------------------------------------

  /** What is picked or pointed at that the renderer's lot lights don't show (a road, a building, a village, a lot not built yet), as ghosts. */
  private litGhosts(): Partial<TownGhosts> | null {
    const s = this.session;
    const t = this.templates;
    if (!s || !t) return null;
    const out: TownGhosts = { roads: [], lots: [], boxes: [] };
    for (const p of [this.selected, this.hovered]) {
      if (!p) continue;
      // (A road picked or pointed at is outlined: the road shows through, never a band over it.)
      if (p.kind === 'road' && s.plan.roads[p.road]) out.roads.push(roadGhost(s.plan.roads[p.road], 'pick'));
      else if (p.kind === 'building' && s.plan.open?.backdrop?.[p.building]) out.lots.push({ ...buildingBox(s.plan.open.backdrop[p.building]), tone: 'new' });
      else if (p.kind === 'village' && s.plan.open?.areas[p.area]) {
        const a = s.plan.open.areas[p.area];
        out.boxes.push({ x: a.x, z: a.z, w: a.w, d: a.d, tone: 'new' });
      } else if (p.kind === 'lot' && s.plan.slots[p.slot] && this.shownPlot(p.slot) === null) out.lots.push({ ...lotBox(t, s.plan.slots[p.slot]), tone: 'new' });
    }
    return out.roads.length || out.lots.length || out.boxes.length ? out : null;
  }

  /** The ghosts drawn again: the edits not yet built, what is picked or pointed at, the tool's own. */
  private redraw(): void {
    const s = this.session;
    const t = this.templates;
    this.drawUses();
    if (!s || !t) {
      this.drawn = null;
      menuScene.ghosts(null);
      return;
    }
    if (!this.editGhostsOf || this.editGhostsOf.settled !== s.settled || this.editGhostsOf.plan !== s.plan) this.editGhostsOf = { settled: s.settled, plan: s.plan, ghosts: editGhosts(s.settled, s.plan, t) };
    // (Drawn again only when they changed: a new ghost makes the renderer draw a whole frame, and the
    // pointer moves many times over while a lot's ghost stays snapped where it is.)
    const lit = this.litGhosts();
    const key = JSON.stringify([lit, this.toolGhosts]);
    if (this.editGhostsOf.ghosts === this.drawn?.edits && key === this.drawn.key) return;
    this.drawn = { edits: this.editGhostsOf.ghosts, key };
    menuScene.ghosts(joinGhosts(joinGhosts(this.editGhostsOf.ghosts, lit), this.toolGhosts));
  }
}

/** No town (the editor has none to edit yet). */
const EMPTY = { name: '', width: 0, depth: 0, slots: [], roads: [], greens: [], households: [] } as unknown as NeighbourhoodDraft;
