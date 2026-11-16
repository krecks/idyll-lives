/**
 * Step 2's Problems list: what is wrong with the town as the player has edited it, read by the same
 * checks the sample tool runs over every generated world (`townCheck.ts` `townProblems`).
 *
 * Those checks read a town file (`CheckedTown`, what `assembleTown` writes), and assembling one
 * means furnishing every house: far too slow after every edit. `checkedTown` writes the part of a
 * town file the checks read straight from the draft, as `assembleTown` would write it (the lots'
 * rectangles, kinds and facings, the roads, the ways out, the water, the parking, the zones, the
 * backdrop, the places and the centre), with nobody living there yet.
 *
 * `townIssues` sorts what the checks find into what blocks the way on to step 3 and what only warns,
 * in the player's words. Blocking: what would leave the sim a town it can't run well, as the plan
 * has it: the roads not one network or no way out, a lot off its road or on a road or the water, a
 * road over water with no deck, a court past its length, a village with no road, fewer than two
 * homes. The rest only warns (a career with nowhere to work, a village of few lots, a road with a
 * hairpin). The checks of how the generator fills a town (how many backdrop buildings an area
 * keeps, the centre the densest) are left out: they hold for generated towns, not edited ones.
 */

import type { Content } from '../../content/content';
import type { RoadPlan } from '../../core/protocol';
import { pointAt, roadExits, lengths } from '../../game/roads';
import { townProblems, type CheckedTown } from '../../game/townCheck';
import { plotRect, type NeighbourhoodDraft, type Templates } from '../../game/town';

/** A problem as the list shows it: what it says, and where it is on the town (if it is somewhere). */
export interface Issue {
  text: string;
  /** The lot it is about (a slot of the draft checked), if one. */
  lot?: number;
  /** Where it is (m), if it is somewhere. */
  at?: { x: number; z: number };
}

/** What `townIssues` finds: `blocking` keeps the player from going on to step 3; `warnings` are worth knowing. */
export interface TownIssues {
  blocking: Issue[];
  warnings: Issue[];
}

/** The town file's kind of a lot (`assembleTown`'s `PlotKind`). */
const plotKind = (s: NeighbourhoodDraft['slots'][number]): string => (s.kind === 'house' ? (s.lot === 'farm' ? 'farm' : 'home') : s.kind === 'shop' ? 'shop' : s.kind === 'outdoor' ? 'countryside' : 'public');

/**
 * The part of `draft`'s town file the checks read (`CheckedTown`), written as `assembleTown` writes
 * it, with nobody living there yet. A few milliseconds, not the second or so a whole file takes.
 */
export function checkedTown(draft: NeighbourhoodDraft, t: Templates): CheckedTown {
  const open = draft.open;
  const W = draft.width;
  const D = draft.depth;
  const plots = draft.slots.map((s) => ({ ...plotRect(t, s), public: s.kind !== 'house', area: s.area ?? 0, kind: plotKind(s) }));
  // (A place's ways out are where its roads, not its footpaths, leave the world; a classic town's, any road's.)
  const exits = open ? roadExits(draft.roads.filter((r) => r.kind !== 'trail'), W, D).map(([x, z]) => ({ x, z, kind: 'road' as const })) : roadExits(draft.roads, W, D);
  return {
    size: 'big',
    width: W,
    depth: D,
    plots,
    exits,
    sims: [],
    households: [],
    roads: draft.roads,
    ...(open ? { areas: open.areas, terrain: { water: open.water }, parking: open.parking } : {}),
    meta: {
      roads: draft.roads,
      greens: draft.greens,
      facing: draft.slots.map((s) => s.facing),
      ...(open
        ? {
            zones: open.zones,
            ...(open.backdrop ? { backdrop: open.backdrop } : {}),
            ...(open.places ? { places: open.places } : {}),
            ...(open.centre ? { centre: open.centre } : {}),
          }
        : {}),
    },
  };
}

/** The checks' lines that keep the player from step 3 (see the top of this file). */
const BLOCKING: readonly RegExp[] = [
  /doesn't join a road out of town$/,
  /^the roads are \d+ networks/,
  /^the way out at .* is off the roads$/,
  / has no way out$/,
  /^no way out of town$/,
  /^a way out of town at .* not on its edge$/,
  /^plot \d+ has \d+ tiles of water$/,
  /^road .* runs over water$/,
  / runs over the water at .* with no bridge$/,
  /'s deck runs .* along it/,
  / runs over the water off its deck at /,
  /'s deck is over no water$/,
  /'s deck is \d+ m long/,
  /'s deck turns /,
  /^plot \d+ off the town$/,
  /^plots \d+ and \d+ overlap$/,
  /^plot \d+ on a road$/,
  /^plot \d+ [\d.]+ m from its road$/,
  /^trail .* runs through plot \d+$/,
  /^two roads called /,
  /^road \d+ .* has no name$/,
  / ends \d+ m past the last road it meets/,
  /^fewer than two homes/,
];

/** The checks of how the generator fills a town, which an edited town needn't keep (see the top of this file). */
const GENERATED_ONLY: readonly RegExp[] = [/ has \d+ backdrop buildings \(\d+ or more\)$/, /^the centre's open land is /, /^the main town has no centre$/];

/** The middle of a road (m). */
function middleOf(r: RoadPlan): { x: number; z: number } {
  const ls = lengths(r.points);
  const [x, z] = pointAt(r.points, ls[ls.length - 1] / 2, ls).p;
  return { x, z };
}

/** A checks' line in the player's words, and where it is: lots by their names, a position taken out of the text and kept. */
export function issueOf(line: string, draft: NeighbourhoodDraft, t: Templates): Issue {
  const slots = draft.slots;
  const lotName = (i: number) => slots[i]?.name ?? `a lot`;
  let lot: number | undefined;
  let at: { x: number; z: number } | undefined;
  let text = line
    .replace(/\bplots (\d+) and (\d+)\b/, (_, a: string, b: string) => {
      lot = Number(a);
      return `${lotName(Number(a))} and ${lotName(Number(b))}`;
    })
    .replace(/\b(outdoor place|plot) (\d+)\b/, (_, what: string, i: string) => {
      lot ??= Number(i);
      return what === 'plot' ? lotName(Number(i)) : `the outdoor place ${lotName(Number(i))}`;
    })
    .replace(/ (?:at|by) (-?[\d.]+), (-?[\d.]+)/, (_, x: string, z: string) => {
      at = { x: Number(x), z: Number(z) };
      return '';
    });
  if (lot !== undefined && slots[lot]) {
    const r = plotRect(t, slots[lot]);
    at = { x: r.x + r.w / 2, z: r.z + r.d / 2 };
  }
  // (Else a road or an area it names.)
  if (!at) {
    const road = draft.roads.find((r) => r.name && text.includes(r.name));
    const area = draft.open?.areas.find((a) => a.name && text.includes(a.name));
    if (road) at = middleOf(road);
    else if (area) at = { x: area.x + area.w / 2, z: area.z + area.d / 2 };
  }
  text = text.replace(/^road (\d+) \((\w+)\) has no name$/, 'a $2 has no name');
  text = text.charAt(0).toUpperCase() + text.slice(1);
  return { text, ...(lot !== undefined && slots[lot] ? { lot } : {}), ...(at ? { at } : {}) };
}

/**
 * What is wrong with `draft` (see the top of this file). `base`: the town as it was laid (its
 * problems, if any, are the generator's: they warn rather than block, the player having made none of them).
 */
export function townIssues(draft: NeighbourhoodDraft, t: Templates, content: Content, base?: TownIssues): TownIssues {
  const careers = content.careerCategories.map((c) => c.id);
  const size = draft.world?.options?.town;
  // (Only the main town's careers are expected, in a town or a city; nothing about how filled it is.)
  const expect = { built: 0, empty: null, meadow: false, quarters: 0, homes: 0, mainLeast: 0, careers: size === 'town' || size === 'city', exits: null };
  const found = townProblems(checkedTown(draft, t), expect, { river: false, careers });
  const homes = draft.slots.filter((s) => s.kind === 'house').length;
  const lines = [...found.hard];
  if (homes < 2) lines.push(`fewer than two homes (${homes})`);
  const had = new Set(base?.blocking.map((i) => i.text) ?? []);
  const out: TownIssues = { blocking: [], warnings: [] };
  for (const line of lines) {
    if (GENERATED_ONLY.some((p) => p.test(line))) continue;
    const issue = issueOf(line, draft, t);
    if (BLOCKING.some((p) => p.test(line)) && !had.has(issue.text)) out.blocking.push(issue);
    else out.warnings.push(issue);
  }
  for (const line of found.soft) out.warnings.push(issueOf(line, draft, t));
  return out;
}
