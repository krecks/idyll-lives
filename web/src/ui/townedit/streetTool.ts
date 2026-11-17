/**
 * The Street tool: draws a street, lane, country road or footpath (`streets.ts` `drawStreet`), as
 * its card (`streetChoice`) asks. Clicks set its points, or a drag draws it freehand. It is drawn
 * as soon as it reaches a road (or the world's edge); Enter or a double click ends it where it is,
 * as a dead end. Backspace takes the last point back, Esc drops it. The street drawn is picked, so
 * its card shows; one split at its bridges, the stretch nearest the last click (`stretchNear`), the
 * card listing the others (`streetChoice.drawn`). What goes along it is the card's choice: homes from the lot mix (`drawStreet`'s
 * fill), the town's other buildings, or both (`buildup.ts` `builtAlong`, in the same edit: one undo
 * takes the street and what stands along it); by default both in the main town, homes elsewhere.
 *
 * As the pointer moves, the street it would make is drawn as a ghost (`streetLive.ts`): blue where
 * it may go, its bridges pale, red where it may not with why under the tools, a mark where it would
 * join a road. Once the pointer rests (`REST` ms), `drawStreet` itself is asked, which also says
 * what is in its way (greyed) and anything the live checks leave to it.
 *
 * The tool's state, apart from the editor (`StreetDrawing`), is plain: the tests drive it.
 */

import type { Edited } from '../../game/edit/edited';
import { builtAlong } from '../../game/edit/buildup';
import { drawStreet, Meeting, streetWidth } from '../../game/edit/streets';
import { lengths, nearest, pointAt, type Pt } from '../../game/roads';
import type { TownGhosts } from '../../render/types';
import { buildingBox, lotBox, roadGhost } from './ghosts';
import StreetCard from './StreetCard.svelte';
import { alongFor, fillsHomes, streetChoice, strokeKind, type Along } from './streetChoice.svelte';
import { liveStreet, newRoads, runsOf, unfinished, verdictOf, warmLive, type LiveStreet, type StrokeKind } from './streetLive';
import type { TownEditor, TownTool } from './tools';
import { sentence } from './words';

/** A press moves less than this (CSS px) and it is a click, not a drag. */
const CLICK = 4;
/** A second click this soon (ms) and this near the first (CSS px) is a double click. */
const DOUBLE_MS = 350;
const DOUBLE_PX = 6;
/** A freehand drag sets a point every this far over the ground (m). */
const FREEHAND = 2;
/** How long the pointer rests (ms) before `drawStreet` itself is asked about the street under it. */
const REST = 140;
/** A join's mark: a square this wide (m). */
const MARK = 4.5;

/** What the street being drawn asks `drawStreet` for. */
export interface StreetAsk {
  kind: StrokeKind;
  clear: boolean;
  /** What goes along it, as the card asks (`auto`: by where it runs, `alongFor`). */
  along: Along;
}

/** What the card asks for now. */
export const askOf = (): StreetAsk => ({ kind: strokeKind(streetChoice.kind), clear: streetChoice.clear, along: streetChoice.along });

/**
 * A street being drawn: the points set, and where the pointer is (the line's last point while it
 * moves). No editor in it: what it would make is asked of it (`live`, `ghosts`, `line`).
 */
export class StreetDrawing {
  points: Pt[] = [];
  cursor: Pt | null = null;

  get drawing(): boolean {
    return this.points.length > 0;
  }

  /** The line as it stands: the points set, and the pointer's (if it is over the ground). */
  line(): Pt[] {
    return this.cursor ? [...this.points, this.cursor] : [...this.points];
  }

  add(p: Pt): void {
    const last = this.points[this.points.length - 1];
    if (last && Math.hypot(p[0] - last[0], p[1] - last[1]) < 0.5) return;
    this.points.push(p);
  }

  /** The last point taken back (Backspace): true if there was one. */
  back(): boolean {
    return this.points.pop() !== undefined;
  }

  clear(): void {
    this.points = [];
    this.cursor = null;
  }
}

/** A join's mark: a small square round it (a lot outline, as the ghosts have them). */
const mark = (at: Pt, ok: boolean) => ({ x: at[0] - MARK / 2, z: at[1] - MARK / 2, w: MARK, d: MARK, tone: ok ? ('new' as const) : ('bad' as const) });

/** Where a street started at `p` would begin: on a road (a T, at its nearest point) or carrying on its dead end; null: in the open. */
export function startAt(draft: Parameters<typeof liveStreet>[0], p: Pt, kind: StrokeKind, meetings = new WeakMap<object, Meeting>()): { at: Pt; road: number; extend: boolean } | null {
  let meet = meetings.get(draft.roads);
  if (!meet || meet.trail !== (kind === 'trail')) meetings.set(draft.roads, (meet = new Meeting(draft.roads, kind === 'trail', streetWidth(kind))));
  const e = meet.snap(p);
  if (e.kind === 'extend') {
    const pts = draft.roads[e.road].points;
    return { at: e.end ? pts[pts.length - 1] : pts[0], road: e.road, extend: true };
  }
  if (e.kind !== 'join') return null;
  const r = draft.roads[e.road];
  const ls = lengths(r.points);
  return { at: pointAt(r.points, meet.index.along(e.road, p[0], p[1]), ls).p, road: e.road, extend: false };
}

/**
 * Of the roads a street was drawn as (`made`: split at its bridges), the one to pick: the nearest `at` (the last
 * click) that isn't a bridge's deck, else the nearest deck. (The first was the stretch before the first bridge,
 * wherever the street was finished.)
 */
export function stretchNear(draft: Pick<Parameters<typeof liveStreet>[0], 'roads'>, made: readonly number[], at: Pt): number | undefined {
  let best: { road: number; deck: boolean; dist: number } | undefined;
  for (const road of made) {
    const r = draft.roads[road];
    if (!r) continue;
    const deck = !!r.bridge;
    const dist = nearest(r.points, at[0], at[1]).dist;
    if (!best || (best.deck && !deck) || (best.deck === deck && dist < best.dist)) best = { road, deck, dist };
  }
  return best?.road;
}

/** The ghosts of a live street: its line in runs (new, bridge, red), its joins' marks; and, from `drawStreet`'s verdict when it is in, the roads it makes and what it clears. */
export function streetGhosts(editor: Pick<TownEditor, 'draft' | 't'>, live: LiveStreet, verdict: Edited | null): Partial<TownGhosts> {
  const draft = editor.draft;
  const ghosts: TownGhosts = { roads: [], lots: [], boxes: [] };
  const refused = verdict && 'refused' in verdict && !unfinished(verdict.refused);
  const made = verdict && 'draft' in verdict && !live.why ? verdict : null;
  if (made && !live.slope) {
    // What it makes, as `drawStreet` makes it (cut at the kerbs, split at its bridges).
    for (const i of newRoads(draft, made.draft)) ghosts.roads.push(roadGhost(made.draft.roads[i], 'new'));
  } else {
    // (The line as the live checks have it: where it is refused, or looks too steep, red.)
    const whole = refused && !live.why ? [{ from: 0, to: Infinity, tone: 'bad' as const }] : live.stretches;
    for (const run of runsOf(live.line, whole)) ghosts.roads.push({ points: run.points, width: live.width, tone: run.tone });
  }
  // What it clears out of its way.
  if (made) {
    for (const i of made.cleared.lots) if (draft.slots[i]) ghosts.lots.push({ ...lotBox(editor.t, draft.slots[i]), tone: 'gone' });
    for (const i of made.cleared.backdrop) if (draft.open?.backdrop?.[i]) ghosts.lots.push({ ...buildingBox(draft.open.backdrop[i]), tone: 'gone' });
  }
  for (const j of live.joins) ghosts.lots.push(mark(j.at, j.ok));
  return ghosts;
}

/** What the street under the pointer would be, in words ("A street of Ashby", "A country road with a 30 m bridge"). */
export function kindLine(draft: Parameters<typeof liveStreet>[0], live: LiveStreet): string {
  const area = live.area >= 0 ? draft.open?.areas[live.area]?.name : undefined;
  const what = live.kind === 'trail' ? 'A footpath' : live.kind === 'country' ? 'A country road' : `A ${live.kind === 'lane' ? 'lane' : 'street'}${area ? ` of ${area}` : ''}`;
  const bridges = live.bridges.length ? ` with ${live.bridges.length === 1 ? `a ${live.bridges[0]} m bridge` : `${live.bridges.length} bridges`}` : '';
  return `${what}${bridges}`;
}

export const streetTool: TownTool = {
  id: 'street',
  label: 'Street',
  icon: 'icon.ui.blueprint',
  does: 'Draw a street, a lane, a country road or a footpath: click points or drag, and end it on a road.',
  stage(editor: TownEditor) {
    const s = new StreetDrawing();
    /** A press under way: where, and whether it became a drag. */
    let press: { x: number; y: number; moved: boolean } | null = null;
    /** The last click let go (for a double click). */
    let lastClick: { at: number; x: number; y: number } | null = null;
    /** The seed of the street's homes and names (a new one after each street drawn). */
    let seed = editor.seed();
    /** What was last worked out, for which line, and `drawStreet`'s verdict on it once the pointer rested. */
    let shown: { key: string; live: LiveStreet; verdict: Edited | null } | null = null;
    let rest: ReturnType<typeof setTimeout> | undefined;
    const meetings = new WeakMap<object, Meeting>();
    // (What the live checks keep of the town made ahead of the first move, not on it.)
    setTimeout(() => warmLive(editor.draft, editor.t), 0);

    const keyOf = (pts: readonly Pt[], ask: StreetAsk) => `${ask.kind}|${ask.clear}|${editor.version}|${pts.map(([x, z]) => `${x.toFixed(1)},${z.toFixed(1)}`).join(';')}`;

    /** The ghosts and the line under the tools for the street as it stands (`verdict`: `drawStreet`'s, once the pointer rested). */
    const show = () => {
      clearTimeout(rest);
      const pts = s.line();
      const ask = askOf();
      if (!s.drawing) {
        shown = null;
        if (!s.cursor) {
          editor.preview(null);
          editor.hint(null);
          return;
        }
        const from = startAt(editor.draft, s.cursor, ask.kind, meetings);
        editor.preview(from ? { lots: [mark(from.at, true)] } : null);
        editor.hint(
          from
            ? `Click to start ${from.extend ? `carrying on ${editor.draft.roads[from.road].name ?? 'this road'}` : `a street off ${editor.draft.roads[from.road].name ?? 'this road'}`}, or drag to draw one.`
            : 'Click on a road to start a street off it, or anywhere to start one there; drag to draw it freehand.',
        );
        return;
      }
      if (pts.length < 2) {
        editor.preview({ lots: [mark(pts[0], true)] });
        editor.hint('Click to set its next point, or drag to draw it.');
        return;
      }
      const key = keyOf(pts, ask);
      if (shown?.key !== key) shown = { key, live: liveStreet(editor.draft, editor.t, pts, ask.kind, ask.clear), verdict: null };
      const { live, verdict } = shown;
      if (live.kind && live.kind !== 'trail') {
        streetChoice.shown = live.kind;
        streetChoice.shownArea = live.area;
      }
      editor.preview(streetGhosts(editor, live, verdict));
      editor.hint(lineOf(live, verdict));
      // (`drawStreet` itself once the pointer rests: too slow for every move on a big town.)
      if (!verdict && !live.why) rest = setTimeout(() => settle(key, pts, ask), REST);
    };

    /** `drawStreet`'s verdict on the line under the pointer, if the pointer is still where it rested. */
    const settle = (key: string, pts: Pt[], ask: StreetAsk) => {
      if (shown?.key !== key) return;
      shown.verdict = verdictOf(editor.draft, editor.t, editor.content, pts, ask.kind, ask.clear, seed);
      editor.preview(streetGhosts(editor, shown.live, shown.verdict));
      editor.hint(lineOf(shown.live, shown.verdict));
    };

    /** The line under the tools: why it may not go, or what it is and what a click would do. */
    const lineOf = (live: LiveStreet, verdict: Edited | null): string => {
      if (live.why) return sentence(live.why);
      if (verdict && 'refused' in verdict && !unfinished(verdict.refused)) return sentence(verdict.refused);
      // (Too steep is the settle's to judge: said, and the street may still be drawn for it to.)
      if (live.slope) return `${sentence(live.slope)} The town would refuse it once brought up to date.`;
      const what = kindLine(editor.draft, live);
      const clears = verdict && 'draft' in verdict ? verdict.cleared.lots.length + verdict.cleared.backdrop.length : 0;
      const also = clears ? `, clearing ${clears === 1 ? 'a lot or building' : `${clears} lots and buildings`} out of its way` : '';
      if (live.reached) return `${what}${also}. ${press ? 'Let go' : 'Click'} to draw it.`;
      const open = live.open ?? (verdict && 'refused' in verdict ? verdict.refused : null);
      return open ? `${what}${also}. ${sentence(open)}` : `${what}${also}. Click to set a point; Enter or a double click ends it here as a dead end.`;
    };

    /** The street drawn as it stands (`asDeadEnd`: ended where it is, else only if it reached a road): true if it was. */
    const finish = (asDeadEnd: boolean): boolean => {
      clearTimeout(rest);
      const pts = s.points;
      if (pts.length < 2) {
        if (asDeadEnd) editor.refuse('a street is drawn with two points or more');
        return false;
      }
      const ask = askOf();
      const live = liveStreet(editor.draft, editor.t, pts, ask.kind, ask.clear);
      if (!asDeadEnd && !live.reached) return false;
      if (live.why) {
        editor.refuse(live.why);
        return false;
      }
      const before = editor.draft;
      const edited = drawn(pts, ask, alongFor(ask.along, live.area));
      if (!editor.apply(edited) || !('draft' in edited)) return false;
      if ('leftOut' in edited && edited.leftOut) editor.refuse(`${edited.leftOut === 1 ? 'a building was' : `${edited.leftOut} buildings were`} left out along it, where the ground is too steep`);
      const made = newRoads(before, edited.draft);
      const road = stretchNear(edited.draft, made, pts[pts.length - 1]);
      s.clear();
      shown = null;
      seed = editor.seed();
      if (road !== undefined) editor.select({ kind: 'road', road });
      // (The card lists the stretches while the draft still has them, by name: `drawnStretches`.)
      streetChoice.drawn = made.map((road) => ({ road, name: edited.draft.roads[road].name ?? '' }));
      editor.preview(null);
      editor.hint(null);
      return true;
    };

    /** The street drawn, and what goes along it: homes by `drawStreet`'s own fill, the town's buildings (and homes with them) by build-up, in the one edit (one undo). */
    const drawn = (pts: Pt[], ask: StreetAsk, along: Exclude<Along, 'auto'>): Edited & { leftOut?: number } => {
      const before = editor.draft;
      const e = drawStreet(before, editor.t, editor.content, { points: pts, kind: ask.kind, clear: ask.clear, fill: fillsHomes(along), seed });
      return along === 'buildings' || along === 'both' ? builtAlong(before, e, editor.t, editor.content, along, seed) : e;
    };

    /** The street being drawn dropped. */
    const drop = () => {
      clearTimeout(rest);
      press = null;
      lastClick = null;
      s.clear();
      shown = null;
      editor.preview(null);
      editor.hint(null);
    };

    const groundPt = (x: number, y: number): Pt | null => {
      const g = editor.ground(x, y);
      return g ? [g.x, g.z] : null;
    };

    return {
      down(x, y) {
        const g = groundPt(x, y);
        if (!g) return false;
        const now = performance.now();
        if (lastClick && s.points.length >= 2 && now - lastClick.at < DOUBLE_MS && Math.hypot(x - lastClick.x, y - lastClick.y) < DOUBLE_PX) {
          lastClick = null;
          press = null;
          finish(true);
          return true;
        }
        press = { x, y, moved: false };
        s.add(g);
        s.cursor = null;
        show();
        return true;
      },
      move(x, y) {
        if (!press) return;
        if (!press.moved && Math.hypot(x - press.x, y - press.y) < CLICK) return;
        press.moved = true;
        const g = groundPt(x, y);
        if (!g) return;
        const last = s.points[s.points.length - 1];
        if (!last || Math.hypot(g[0] - last[0], g[1] - last[1]) >= FREEHAND) s.add(g);
        s.cursor = g;
        show();
      },
      up() {
        const p = press;
        press = null;
        if (!p) return;
        if (p.moved && s.cursor) s.add(s.cursor);
        lastClick = p.moved ? null : { at: performance.now(), x: p.x, y: p.y };
        // (A click or a drag that reached a road ends the street there.)
        if (!finish(false)) show();
      },
      hover(x, y) {
        s.cursor = groundPt(x, y);
        show();
      },
      leave() {
        s.cursor = null;
        if (s.drawing) show();
        else {
          clearTimeout(rest);
          editor.preview(null);
          editor.hint(null);
        }
      },
      cancel: drop,
      key(e) {
        if (!s.drawing) return false;
        if (e.key === 'Escape') {
          drop();
          return true;
        }
        if (e.key === 'Backspace' || e.key === 'Delete') {
          if (press) return true;
          s.back();
          shown = null;
          show();
          return true;
        }
        if (e.key === 'Enter') {
          if (!press) finish(true);
          return true;
        }
        return false;
      },
    };
  },
  card: StreetCard,
};
