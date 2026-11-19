// Step 2's street edits (`game/edit/streets.ts`) on a small hand-made town: a high street across the
// world with a lane off it, a river south of it with a far lane over the water reached round its
// end. A street snaps to the road it starts by (a T at the kerb) or carries on a dead end; meets a
// road at 35° or more (square within 15°), 14 m from another junction (sliding along the road to where
// there is room, no nearer its ends than 6 m); is refused where it bends
// tighter than 9 m; crosses a 30 m river on a straight bridge and is refused over 60 m; clears what
// is in its way; is named apart from every name in town. A road removed takes what it leaves
// joined to nothing, keeps one network, and gives the water under a bridge back. The Street tool's
// magnet; a street or a lane asked for outside the town; the first street on bare land. `bun test`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { Content } from '../src/content/content';
import type { RoadPlan } from '../src/core/protocol';
import { drawStreet, madeKind, Meeting, removeRoad, rename, streetsBare, whatGoes } from '../src/game/edit/streets';
import type { Edited } from '../src/game/edit/edited';
import { checkedTown, editedProblems } from '../src/game/edit/checked';
import { DRAWN_SPAN } from '../src/game/bridges';
import { deckTiles } from '../src/game/macro';
import { blankLand, LAND_CELL } from '../src/game/land';
import { wetTile, wetTiles } from '../src/game/landEdit';
import { rle, unrle } from '../src/game/landMasks';
import { townProblems } from '../src/game/townCheck';
import { whatGoesLine } from '../src/ui/townedit/words';
import { lengths, nearest, pointAt, type Pt } from '../src/game/roads';
import type { NeighbourhoodDraft, PlotSlot, Templates } from '../src/game/town';

const W = 400;
const D = 400;
/** The high street's middle (z) and the river's banks (z), west of `RIVER_END` (x). */
const MAIN_Z = 100;
const RIVER_END = 340;
const FAR_Z = 290;
/** Where the lane leaves the high street (x). */
const LANE_X = 170;

const line = (x0: number, z0: number, x1: number, z1: number): Pt[] => {
  const n = Math.max(1, Math.ceil(Math.hypot(x1 - x0, z1 - z0)));
  return Array.from({ length: n + 1 }, (_, i): Pt => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
};
const rect = (x0: number, z0: number, x1: number, z1: number): Pt[] => [
  [x0, z0],
  [x1, z0],
  [x1, z1],
  [x0, z1],
];

/** A house lot, `w` × `d`, south of the high street (its street to the north: facing 0), or north of the far lane (facing 2). */
function lot(index: number, x: number, z: number, facing: 0 | 2, name: string): PlotSlot {
  return { index, kind: 'house', template: 'house', name, x, z, facing, lot: 'medium', w: 14, d: 18 };
}

/**
 * The town: the high street (0) out of the world both ways; a lane off it at x 170 (1), a 28 m court, a parking space by it;
 * the far lane north of the river (2), a dead end at x 40, joined at x 380 to the road round the
 * river's end (3). Houses along the high street's south side and the far lane's north side. The
 * river runs from the west edge to x 340 between z 200 and 200 + `river`.
 */
function town(river = 30): NeighbourhoodDraft {
  const roads: RoadPlan[] = [
    { points: line(0, MAIN_Z, W, MAIN_Z), width: 7, ends: ['out', 'out'], kind: 'street', area: 0, name: 'High Street' },
    { points: line(LANE_X, MAIN_Z + 2.15, LANE_X, 130), width: 7, ends: ['join', 'dead'], kind: 'lane', area: 0, name: 'Mill Lane' },
    { points: line(40, FAR_Z, 380, FAR_Z), width: 7, ends: ['dead', 'join'], kind: 'country', name: 'Far Lane' },
    { points: line(380, MAIN_Z + 2.15, 380, FAR_Z), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Round Road' },
  ];
  const slots: PlotSlot[] = [];
  for (let x = 60; x <= 320; x += 40) slots.push(lot(slots.length, x, MAIN_Z + 6, 0, `${slots.length * 2 + 1} High Street`));
  for (let x = 60; x <= 320; x += 40) slots.push(lot(slots.length, x, FAR_Z - 6 - 18, 2, `${slots.length} Far Lane`));
  const water = new Uint8Array(W * D);
  for (let z = 200; z < 200 + river; z++) for (let x = 0; x < RIVER_END; x++) water[z * W + x] = 1;
  return {
    name: 'Ashby',
    width: W,
    depth: D,
    slots,
    roads,
    greens: [],
    households: [],
    open: {
      areas: [{ name: 'Ashby', kind: 'hamlet', x: 50, z: 50, w: 300, d: 200 }],
      zones: [{ kind: 'water', poly: rect(0, 200, RIVER_END, 200 + river) }],
      water: rle(water),
      pads: {},
      parking: [{ x: LANE_X + 7, z: 120, yaw: 0, area: 0 }],
      places: [],
      backdrop: [],
    },
  };
}

const t: Templates = { plot: { width: 14, depth: 18 }, houses: [], parks: [] };
const content = { names: { streets: ['Orchard Way', 'Church Road', 'Kiln Lane', 'Station Road', 'Brook Street', 'Elm Close'] } } as unknown as Content;

/** The street drawn through `points` on `draft`. */
const draw = (draft: NeighbourhoodDraft, points: Pt[], more: Partial<{ kind: RoadPlan['kind'] & string; clear: boolean; seed: number }> = {}) => drawStreet(draft, t, content, { points, kind: more.kind ?? 'street', clear: more.clear ?? true, fill: false, seed: more.seed ?? 7 });
const made = (e: Edited) => {
  if ('refused' in e) throw new Error(`refused: ${e.refused}`);
  return e;
};
const refusal = (e: Edited) => ('refused' in e ? e.refused : '');
/** Step 2's Problems list's checks of a draft (`checked.ts` `editedProblems`), no careers asked of it. */
const problemsOf = (draft: NeighbourhoodDraft) => editedProblems(draft, t, { ...content, careerCategories: [] } as unknown as Content);

/** The angle (degrees, 0..90) a road's end meets road `o` at. */
function meeting(r: RoadPlan, end: 0 | 1, o: RoadPlan): number {
  const pts = end ? r.points : [...r.points].reverse();
  const L = lengths(pts).at(-1)!;
  const a = pointAt(pts, L - 3).p;
  const e = pts[pts.length - 1];
  const v = [e[0] - a[0], e[1] - a[1]];
  const tq = pointAt(o.points, nearest(o.points, e[0], e[1]).s).t;
  const cos = Math.abs(v[0] * tq[0] + v[1] * tq[1]) / Math.hypot(v[0], v[1]);
  return (Math.acos(Math.min(1, cos)) * 180) / Math.PI;
}

/** Whether `roads` are one network, joined as the sim joins them, with a way out. */
function oneNetwork(roads: readonly RoadPlan[]): boolean {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  roads.forEach((r, i) => {
    for (const p of [r.points[0], r.points[r.points.length - 1]]) roads.forEach((o, j) => j !== i && nearest(o.points, p[0], p[1]).dist <= Math.max(o.width, r.width) / 2 + 1 && (parent[find(i)] = find(j)));
  });
  return new Set(roads.map((_, i) => find(i))).size === 1 && roads.some((r) => r.ends.includes('out'));
}

describe('drawing a street', () => {
  test('its start snaps to a road within 10 m of its edge, a T beginning at the kerb', () => {
    const { draft } = made(draw(town(), [[100, MAIN_Z + 3.5 + 9], [100, 140]]));
    const r = draft.roads.at(-1)!;
    expect(r.ends).toEqual(['join', 'dead']);
    // At the high street's kerb (its middle 3.5 m off, the sidewalk 1.35 m of that).
    expect(r.points[0][1]).toBeGreaterThanOrEqual(MAIN_Z + 3.5 - 1.35 - 0.01);
    expect(r.points[0][1]).toBeLessThan(MAIN_Z + 3.5);
    expect(Math.abs(r.points[0][0] - 100)).toBeLessThan(1);
    expect(r.kind).toBe('street');
    expect(r.area).toBe(0);
    // Further off, it joins nothing.
    expect(refusal(draw(town(), [[100, MAIN_Z + 3.5 + 15], [100, 150]]))).toBe('it joins none of the roads');
  });

  test('a stroke that reaches a road ends there, joining it', () => {
    // From the high street south, round and back north across it: it stops at its kerb.
    const { draft } = made(draw(town(), [[60, MAIN_Z + 6], [65, 135], [80, 145], [95, 135], [100, MAIN_Z - 30]]));
    const r = draft.roads.at(-1)!;
    expect(r.ends).toEqual(['join', 'join']);
    // Its end on the high street's kerb, not across it.
    expect(r.points.at(-1)![1]).toBeGreaterThan(MAIN_Z);
    expect(draft.roads.length).toBe(5);
  });

  test("a dead end it starts by is carried on, the road keeping its name", () => {
    const before = town();
    const { draft } = made(draw(before, [[LANE_X, 133], [LANE_X + 5, 150], [LANE_X + 25, 165], [LANE_X + 45, 150], [LANE_X + 50, 125], [LANE_X + 50, MAIN_Z + 6]]));
    expect(draft.roads.length).toBe(before.roads.length);
    const lane = draft.roads[1];
    expect(lane.name).toBe('Mill Lane');
    expect(lane.ends).toEqual(['join', 'join']);
    expect(lengths(lane.points).at(-1)!).toBeGreaterThan(80);
  });

  test('it meets a road at 35° or more, and square within 15° of it', () => {
    const square = made(draw(town(), [[130, 140], [128, MAIN_Z + 5]]));
    const r = square.draft.roads.at(-1)!;
    expect(meeting(r, r.ends[0] === 'join' ? 0 : 1, square.draft.roads[0])).toBeGreaterThan(89.5);
    // Round from the high street and back onto it at about 25° over its last 13 m: turned to 35°.
    const slant = made(draw(town(), [[60, MAIN_Z + 6], [60, 135], [95, 140], [118, 113], [130, MAIN_Z + 7.5]]));
    const s = slant.draft.roads.at(-1)!;
    const angle = meeting(s, 1, slant.draft.roads[0]);
    expect(angle).toBeGreaterThanOrEqual(34.5);
    expect(angle).toBeLessThan(75);
    // A long way at a slant alongside it is refused.
    expect(refusal(draw(town(), [[10, 128], [130, MAIN_Z + 6]]))).toContain('alongside');
  });

  test('a junction keeps 14 m from the next: one drawn nearer slides along the road to where there is room, else it is refused', () => {
    // North off the high street, 10 m along from the lane's junction: slid 4 m on.
    const slid = made(draw(town(), [[LANE_X + 10, MAIN_Z - 6], [LANE_X + 11, 62]])).draft.roads.at(-1)!;
    expect(slid.points[0][0]).toBeGreaterThanOrEqual(LANE_X + 14 - 0.5);
    expect(slid.points[0][0]).toBeLessThan(LANE_X + 16);
    made(draw(town(), [[LANE_X + 26, MAIN_Z - 6], [LANE_X + 26, 62]]));
    // (Between two lanes 24 m apart nowhere along is room.)
    const crowded = town();
    crowded.roads.push({ points: line(LANE_X + 24, MAIN_Z - 2.15, LANE_X + 24, 70), width: 7, ends: ['join', 'dead'], kind: 'lane', area: 0, name: 'Bell Lane' });
    expect(refusal(draw(crowded, [[LANE_X + 12, MAIN_Z - 6], [LANE_X + 12, 62]]))).toContain('within 14 m of another junction');
  });

  test('joinTo meets a road no nearer its ends than 6 m, sliding along it rather than refusing; the room it looks for, 16 m either way at most', () => {
    const draft = town();
    const meet = new Meeting(draft.roads, false, 7);
    // (Onto Mill Lane, 2 m short of its dead end at z 130: it meets it 6 m short.)
    const end = meet.joinTo(line(LANE_X + 40, 128, LANE_X + 4, 128), 1);
    if (typeof end === 'string') throw new Error(end);
    expect(end.at[1]).toBeCloseTo(124, 1);
    // (Onto Round Road 3 m from its junction with the high street: past the 6 m, and slid to 14 m from the junction.)
    const near = meet.joinTo(line(300, 120, 376, MAIN_Z + 5), 3);
    if (typeof near === 'string') throw new Error(near);
    expect(near.at[0]).toBeCloseTo(380, 1);
    expect(near.at[1]).toBeGreaterThanOrEqual(MAIN_Z + 14 - 0.5);
    expect(near.at[1]).toBeLessThanOrEqual(MAIN_Z + 2.15 + 6 + 16);
    // (A road shorter than twice that: too near its end wherever it is met.)
    const stub = new Meeting([...draft.roads, { points: line(200, 300, 210, 300), width: 7, ends: ['dead', 'dead'], kind: 'street', name: 'Stub' }], false, 7);
    expect(stub.joinTo(line(205, 340, 205, 304), 4)).toBe('it meets Stub too near its end');
  });

  test("the magnet: beside the nearest road (2 m past its edge), a dead end over a road less than 10 m nearer, the world's edge; nothing out of reach, nor where the point snaps already", () => {
    const draft = town();
    const meet = new Meeting(draft.roads, false, 7);
    const road = meet.magnet([120, 80], 28, W, D)!;
    expect(road.kind).toBe('road');
    expect(road.road).toBe(0);
    expect(road.at[0]).toBeCloseTo(120, 6);
    expect(road.at[1]).toBeCloseTo(MAIN_Z - 5.5, 6);
    expect(meet.magnet([LANE_X + 3, 146], 28, W, D)).toEqual({ at: [LANE_X, 130], road: 1, kind: 'dead' });
    expect(meet.magnet([395, 30], 28, W, D)).toEqual({ at: [W, 30], road: -1, kind: 'edge' });
    expect(meet.magnet([250, 160], 28, W, D)).toBeNull();
    expect(meet.magnet([120, 80], 10, W, D)).toBeNull();
    // (Within 10 m of the high street's edge a street's end snaps to it as it is.)
    expect(meet.magnet([120, 88], 28, W, D)).toBeNull();
    // (What `keep` leaves out does not pull.)
    expect(meet.magnet([120, 80], 28, W, D, (_, r) => r !== 0)).toBeNull();
    // (A footpath is pulled to a footpath's dead end, a street not.)
    const paths: RoadPlan[] = [...draft.roads, { points: line(250, 40, 250, 70), width: 2, ends: ['dead', 'dead'], kind: 'trail' }];
    expect(new Meeting(paths, true, 2).magnet([253, 24], 28, W, D)?.kind).toBe('dead');
    expect(new Meeting(paths, false, 7).magnet([253, 24], 28, W, D)?.kind).not.toBe('dead');
  });

  test('a street or a lane asked for stays one outside the town, with no area; by where it runs it is a country road there', () => {
    const draft = town();
    // (East of the town's box and more than 30 m from it.)
    const out = line(390, 120, 390, 280);
    expect(madeKind(draft, out, 'street')).toEqual({ kind: 'country' });
    expect(madeKind(draft, out, 'street', true)).toEqual({ kind: 'street' });
    expect(madeKind(draft, out, 'lane', true)).toEqual({ kind: 'lane' });
    expect(madeKind(draft, out, 'country', true)).toEqual({ kind: 'country' });
    expect(madeKind(draft, line(100, 120, 100, 150), 'lane', true)).toEqual({ kind: 'lane', area: 0 });
    expect(madeKind(draft, out, 'trail')).toEqual({ kind: 'trail' });
  });

  test('with no road a car drives, the first street runs from the edge of the world to its edge', () => {
    const bare = { ...town(0), roads: [], slots: [] };
    expect(streetsBare(bare)).toBe(true);
    expect(streetsBare({ roads: [{ points: line(0, 0, 10, 0), width: 2, ends: ['dead', 'dead'], kind: 'trail' }] })).toBe(true);
    expect(streetsBare(town())).toBe(false);
    expect(refusal(draw(bare, [[60, 150], [100, 150]]))).toBe('it joins none of the roads');
    // (One from the edge into the land is a dead end, as long as it is drawn: the player's to carry on.)
    expect(made(draw(bare, [[0, 150], [200, 150]])).draft.roads[0].ends).toEqual(['out', 'dead']);
    const first = made(draw(bare, [[0, 150], [W, 150]])).draft.roads;
    expect(first).toHaveLength(1);
    expect(first[0].ends).toEqual(['out', 'out']);
  });

  test('a bend tighter than 9 m is refused', () => {
    expect(refusal(draw(town(), [[60, MAIN_Z + 6], [60, 140], [66, 140], [66, 125]]))).toBe('it bends tighter than 9 m');
  });

  test('a dead end may reach past a court: a street or a lane as drawn, a country road made a street of no area', () => {
    const street = made(draw(town(), [[100, MAIN_Z + 6], [100, 170]])).draft.roads.at(-1)!;
    expect(street).toMatchObject({ kind: 'street', area: 0, ends: ['join', 'dead'] });
    // (South off Far Lane, out of the town's box: a country road, but no taxi turns round in one.)
    const out = made(draw(town(), [[200, FAR_Z + 6], [200, 380]])).draft;
    const road = out.roads.at(-1)!;
    expect(road.kind).toBe('street');
    expect(road.area).toBeUndefined();
    expect(road.ends).toEqual(['join', 'dead']);
    // (Nothing in the Problems list but the town's own: Far Lane, a country road, ends far past the roads it meets.)
    expect(problemsOf(out).hard.filter((h) => !h.startsWith('Far Lane ends'))).toEqual([]);
    // (A short one stays a country road.)
    expect(made(draw(town(), [[200, FAR_Z + 6], [200, FAR_Z + 40]])).draft.roads.at(-1)!.kind).toBe('country');
  });

  test("what is in its way is cleared, or the street refused", () => {
    const e = made(draw(town(), [[100, MAIN_Z + 6], [100, 140]]));
    // The lot at x 100 south of the high street.
    expect(e.cleared.lots).toEqual([1]);
    expect(e.draft.slots.length).toBe(town().slots.length - 1);
    expect(e.draft.slots.map((s) => s.index)).toEqual(e.draft.slots.map((_, i) => i));
    expect(refusal(draw(town(), [[100, MAIN_Z + 6], [100, 140]], { clear: false }))).toBe('1 lot is in its way');
  });

  test('a 30 m river is bridged by a straight country road with one deck, the street split round it', () => {
    const before = town(30);
    const { draft } = made(draw(before, [[120, MAIN_Z + 6], [120, FAR_Z - 6]]));
    const added = draft.roads.slice(before.roads.length);
    expect(added.length).toBe(3);
    const bridge = added.find((r) => r.bridge)!;
    expect(bridge.kind).toBe('country');
    const [s0, s1] = bridge.bridge!;
    expect(s1 - s0).toBeGreaterThan(30);
    expect(s1 - s0).toBeLessThanOrEqual(45);
    // Straight: every point on the line between its ends.
    const [a, b] = [bridge.points[0], bridge.points.at(-1)!];
    for (const p of bridge.points) expect(Math.abs((p[0] - a[0]) * (b[1] - a[1]) - (p[1] - a[1]) * (b[0] - a[0])) / Math.hypot(b[0] - a[0], b[1] - a[1])).toBeLessThan(0.05);
    // 12 m before the bank and after it.
    expect(Math.min(a[1], b[1])).toBeLessThanOrEqual(200 - 12 + 0.5);
    expect(Math.max(a[1], b[1])).toBeGreaterThanOrEqual(230 + 12 - 0.5);
    // The street before and after it, joined end to end; nothing else over the water.
    const [first, , after] = [draft.roads[before.roads.length], bridge, added.find((r) => r !== bridge && r !== draft.roads[before.roads.length])!];
    expect(first.ends).toEqual(['join', 'join']);
    expect(after.ends).toEqual(['join', 'join']);
    const wet = unrle(draft.open!.water, W * D);
    const before0 = unrle(before.open!.water, W * D);
    for (const r of [first, after]) for (const [x, z] of r.points) expect(before0[Math.floor(z) * W + Math.floor(x)]).toBe(0);
    // The deck walked dry, the rest of the river not.
    expect(wet[215 * W + 120]).toBe(0);
    expect(wet[215 * W + 60]).toBe(1);
    // Names: all different, none the town had.
    const names = draft.roads.map((r) => r.name);
    expect(new Set(names).size).toBe(names.length);
    expect(bridge.name).toMatch(/Bridge$/);
    expect(oneNetwork(draft.roads)).toBe(true);
  });

  test('a pond too small for the water zones is bridged over water the checks see (the land’s)', () => {
    // A 4 m pond south of the high street at x 120: on the land (two samples by two) and in the water's tiles made from
    // it, but under the 4 samples a water zone needs (`landZones`), so in no zone, as on a generated land.
    const before = town(30);
    const land = blankLand(W, D, 'follow');
    for (let j = 68; j <= 69; j++) for (let i = 60; i <= 61; i++) land.wl[j * land.cols + i] = 50;
    const water = unrle(before.open!.water, W * D);
    for (let z = 0; z < D; z++) for (let x = 0; x < W; x++) if (wetTile(land, x, z)) water[z * W + x] = 1;
    const ponded: NeighbourhoodDraft = { ...before, open: { ...before.open!, land, water: rle(water) } };
    const { draft } = made(draw(ponded, [[120, MAIN_Z + 6], [120, FAR_Z - 6]]));
    const decks = draft.roads.filter((r) => r.bridge);
    expect(decks.length).toBe(2);
    const deckLines = (town: NeighbourhoodDraft) => townProblems(checkedTown(town, t)).hard.filter((h) => /deck|water/.test(h));
    expect(deckLines(draft)).toEqual([]);
    // (Checked against the water zones alone, the pond's deck was over no water: what blocked the way on.)
    expect(deckLines({ ...draft, open: { ...draft.open!, land: undefined } })).toEqual([`${decks.find((r) => r.points.every((q) => q[1] < 180))!.name}'s deck is over no water`]);
  });

  test('a line that only clips the water is bridged through the water’s middle, every deck over water', () => {
    // One wet tile by the line drawn from the high street to the far lane (north of the river's reach of one bridge),
    // tried round the line: where the bridge made straight along it would miss the tile (no deck), the straight is
    // laid through the middle of the water the line crosses (`bridges.ts` `bridgeFits`) instead of refusing it.
    const before = town(30);
    const said = new Map<string, number>();
    let clipped = 0;
    for (let z = 135; z <= 142; z++) {
      for (let x = 122; x <= 132; x++) {
        const water = unrle(before.open!.water, W * D);
        water[z * W + x] = 1;
        const e = draw({ ...before, open: { ...before.open!, water: rle(water) } }, [[120, MAIN_Z + 6], [160, FAR_Z - 6]]);
        const k = 'refused' in e ? e.refused : 'drawn';
        said.set(k, (said.get(k) ?? 0) + 1);
        if ('refused' in e) continue;
        for (const r of e.draft.roads) {
          if (!r.bridge) continue;
          expect(r.bridge[1]).toBeGreaterThan(r.bridge[0]);
          let over = 0;
          deckTiles(r, r.bridge, W, D, (tile) => (over += water[tile]));
          expect(over).toBeGreaterThan(0);
        }
        // (The tile's own crossing, where the line drawn crosses it.)
        if (e.draft.roads.filter((r) => r.bridge).length === 2) clipped++;
      }
    }
    expect([...said.keys()]).toEqual(['drawn']);
    expect(clipped).toBeGreaterThan(0);
  });

  test(`a 60 m crossing is bridged (a drawn street's deck is ${DRAWN_SPAN} m at most); an 80 m one is refused`, () => {
    const { draft } = made(draw(town(60), [[120, MAIN_Z + 6], [120, FAR_Z + 20 - 6]]));
    const deck = draft.roads.find((r) => r.bridge)!.bridge!;
    expect(deck[1] - deck[0]).toBeCloseTo(64, 0);
    expect(refusal(draw(town(80), [[120, MAIN_Z + 6], [120, FAR_Z - 6]]))).toContain('the water is 80 m across');
  });

  test('a footpath does not cross the water', () => {
    expect(refusal(draw(town(30), [[120, MAIN_Z + 6], [120, FAR_Z - 6]], { kind: 'trail' }))).toBe('a footpath does not cross the water');
  });

  test('new names are none the town has, and differ street to street', () => {
    const first = made(draw(town(), [[100, MAIN_Z + 6], [100, 140]], { seed: 3 })).draft;
    const second = made(draw(first, [[260, MAIN_Z + 6], [260, 140]], { seed: 3 })).draft;
    const names = second.roads.map((r) => r.name);
    expect(new Set(names).size).toBe(names.length);
    expect(names).not.toContain('Ashby');
  });

  test('the same stroke gives the same town', () => {
    const a = draw(town(), [[120, MAIN_Z + 6], [120, FAR_Z - 6]], { seed: 11 });
    const b = draw(town(), [[120, MAIN_Z + 6], [120, FAR_Z - 6]], { seed: 11 });
    expect(JSON.stringify(a)).toBe(JSON.stringify(b));
  });
});

describe('a street over a footpath and a river', () => {
  /** A footpath from the high street at x `from` south to `bankZ`, then east along the bank to a jetty (an outdoor place) at x `to`. */
  const withTrail = (draft: NeighbourhoodDraft, bankZ: number, from = 60, to = 200): NeighbourhoodDraft => {
    const jetty: PlotSlot = { index: draft.slots.length, kind: 'outdoor', template: 'jetty', name: 'Ashmouth Fishing Jetty', x: to, z: bankZ - 12, facing: 0, lot: 'medium', w: 12, d: 10 };
    const trail: RoadPlan = { points: [...line(from, MAIN_Z + 3.5, from, bankZ), ...line(from, bankZ, to - 2, bankZ).slice(1)], width: 2, ends: ['join', 'dead'], kind: 'trail', area: 0, name: 'Ashmouth Fishing Jetty Trail' };
    return { ...draft, slots: [...draft.slots, jetty], roads: [...draft.roads, trail] };
  };
  /** Each footpath piece's ends that are not the footpath's own: within the street's width (less half a metre) of a street's middle, in a T. */
  const cutEnds = (draft: NeighbourhoodDraft, name: string, was: RoadPlan) => {
    const streets = draft.roads.filter((r) => r.kind !== 'trail');
    const own = [was.points[0], was.points[was.points.length - 1]];
    return draft.roads
      .filter((r) => r.kind === 'trail' && r.name?.startsWith(name))
      .flatMap((r) => r.ends.flatMap((e, k) => (e === 'join' ? [k ? r.points[r.points.length - 1] : r.points[0]] : [])))
      .filter(([x, z]) => !own.some((p) => Math.hypot(p[0] - x, p[1] - z) < 0.05))
      .map(([x, z]) => Math.min(...streets.map((o) => nearest(o.points, x, z).dist - o.width / 2)));
  };
  const hardOf = (draft: NeighbourhoodDraft) => problemsOf(draft).hard.filter((h) => !h.startsWith('Far Lane ends'));

  test('a footpath it crosses is cut, each piece ending in a T just inside its edge; one network, no new problem', () => {
    const before = withTrail(town(30), 160);
    const trail = before.roads.length - 1;
    const e = made(draw(before, [[120, MAIN_Z + 6], [120, FAR_Z - 6]]));
    const { draft } = e;
    // The trail's piece with its jetty end keeps its index and name; the piece from the high street is named after it.
    expect(draft.roads[trail].name).toBe('Ashmouth Fishing Jetty Trail');
    expect(draft.roads[trail].ends).toEqual(['join', 'dead']);
    expect(draft.roads[trail].points.at(-1)).toEqual(before.roads[trail].points.at(-1)!);
    const pieces = draft.roads.filter((r) => r.name?.startsWith('Ashmouth Fishing Jetty Trail'));
    expect(pieces.map((r) => r.name).sort()).toEqual(['Ashmouth Fishing Jetty Trail', 'Ashmouth Fishing Jetty Trail 2']);
    for (const d of cutEnds(draft, 'Ashmouth Fishing Jetty Trail', before.roads[trail])) {
      expect(d).toBeLessThan(-0.25);
      expect(d).toBeGreaterThan(-1);
    }
    // (The street's own pieces, not the footpath's.)
    expect(e.made!.every((i) => draft.roads[i].kind !== 'trail')).toBe(true);
    expect(e.made).toHaveLength(3);
    expect(oneNetwork(draft.roads)).toBe(true);
    expect(hardOf(draft)).toEqual(hardOf(before));
  });

  test('a footpath along the bank, under the deck, passes under it whole', () => {
    const before = withTrail(town(30), 199);
    const e = made(draw(before, [[120, MAIN_Z + 6], [120, FAR_Z - 6]]));
    expect(e.draft.roads.filter((r) => r.kind === 'trail')).toEqual([before.roads[before.roads.length - 1]]);
  });

  test("the user's stroke: from the high street over the jetty trail and a 30 m painted river in a dip, ending out in the open; bridged, a long dead end, nothing hard in the Problems list", () => {
    // The far bank open land (no Far Lane, no Round Road), the town's box north of the river; the land 10 m up, the
    // river painted 5 m below it (its bed 2 m deeper), a low shelf 3 m wide either side, the banks rising from it over 5 m.
    const base = town(30);
    const land = blankLand(W, D, 'follow');
    for (let j = 0; j < land.rows; j++) {
      for (let i = 0; i < land.cols; i++) {
        const z = j * LAND_CELL;
        const off = z < 200 ? 200 - z : z >= 230 ? z - 229 : 0;
        const k = j * land.cols + i;
        if (!off && i * LAND_CELL < RIVER_END) [land.h[k], land.wl[k]] = [300, 500];
        else land.h[k] = Math.round(525 + Math.max(0, Math.min(1, (off - 3) / 5)) * 475);
      }
    }
    const water = wetTiles(land, W, D);
    let draft: NeighbourhoodDraft = {
      ...base,
      roads: base.roads.slice(0, 2),
      slots: base.slots.filter((s) => s.z < 150),
      open: { ...base.open!, areas: [{ ...base.open!.areas[0], d: 120 }], water: rle(water), land },
    };
    draft = withTrail(draft, 190, 60, 200);
    const e = made(draw(draft, [[130, MAIN_Z + 6], [128, 190], [122, 260], [120, 330]]));
    const after = e.draft;
    const bridge = after.roads.find((r) => r.bridge)!;
    const [s0, s1] = bridge.bridge!;
    // Over the water and on over the bank's dip (the land under it more than 3 m under its ends'), no longer than 70 m.
    expect(s1 - s0).toBeGreaterThan(36);
    expect(s1 - s0).toBeLessThanOrEqual(DRAWN_SPAN);
    // The long dead end on the far bank: a street of no area (no taxi turns round in a country road).
    const far = e.made!.map((i) => after.roads[i]).find((r) => r.ends.includes('dead'))!;
    expect(far.kind).toBe('street');
    expect(far.area).toBeUndefined();
    // The trail cut, in T's either side of the street.
    expect(after.roads.filter((r) => r.name?.startsWith('Ashmouth Fishing Jetty Trail')).length).toBe(2);
    const cuts = cutEnds(after, 'Ashmouth Fishing Jetty Trail', draft.roads[draft.roads.length - 1]);
    expect(cuts).toHaveLength(2);
    for (const d of cuts) expect(d).toBeLessThan(0);
    expect(oneNetwork(after.roads)).toBe(true);
    expect(hardOf(after).filter((h) => /bridge|deck|court|past the last road|network|water|way out/.test(h))).toEqual([]);
    expect(hardOf(after)).toEqual(hardOf(draft));
  });

  test('a freehand bend over the river, a 60 m crossing and a diagonal too long at its slant, squared, are all bridged', () => {
    // (Bent 25 m across the river: the straight laid through the water's middle.)
    const bent = made(draw(town(30), [[100, MAIN_Z + 6], [100, 185], [125, 245], [125, FAR_Z - 6]])).draft;
    expect(bent.roads.filter((r) => r.bridge)).toHaveLength(1);
    const wide = made(draw(town(60), [[120, MAIN_Z + 6], [120, FAR_Z + 14]])).draft;
    expect(wide.roads.filter((r) => r.bridge)).toHaveLength(1);
    // (50 m of water crossed at about 45°: 70 m and more along the line drawn; the bridge turned squarer.)
    const diagonal = made(draw(town(50), [[60, MAIN_Z + 6], [250, FAR_Z - 6]])).draft;
    const b = diagonal.roads.find((r) => r.bridge)!;
    expect(b.bridge![1] - b.bridge![0]).toBeLessThanOrEqual(DRAWN_SPAN);
    const [p, q] = [b.points[0], b.points.at(-1)!];
    const slant = (Math.atan2(Math.abs(q[0] - p[0]), Math.abs(q[1] - p[1])) * 180) / Math.PI;
    expect(slant).toBeLessThan(43 - 5);
    expect(oneNetwork(diagonal.roads)).toBe(true);
  });
});

describe('removing a road', () => {
  test('the roads it leaves joined to nothing go with it, and their lots', () => {
    const before = town();
    const goes = whatGoes(before, t, 3);
    if ('refused' in goes) throw new Error(goes.refused);
    // The road round the river's end, and the far lane it alone joined to the rest.
    expect(goes.roads).toEqual([2, 3]);
    expect(goes.lots).toEqual([7, 8, 9, 10, 11, 12, 13]);
    const { draft, cleared } = made(removeRoad(before, t, 3));
    expect(cleared.lots).toEqual(goes.lots);
    expect(draft.roads.map((r) => r.name)).toEqual(['High Street', 'Mill Lane']);
    expect(draft.slots.length).toBe(7);
    expect(oneNetwork(draft.roads)).toBe(true);
  });

  test('refused if no road out of the world is left', () => {
    expect(refusal(removeRoad(town(), t, 0))).toContain('no road would lead out of the world');
  });

  test('refused if it would leave a country road a dead end longer than a court (no taxi turns round there)', () => {
    // A country road from the road round the river's end west to x 200, joined there by a road up from the high street.
    const all = town();
    const roads: RoadPlan[] = [
      ...all.roads,
      { points: line(376.5, 150, 200, 150), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Link Road' },
      { points: line(200, MAIN_Z + 3.5, 200, 153.5), width: 7, ends: ['join', 'join'], kind: 'country', name: 'West Road' },
    ];
    const linked = { ...all, roads };
    expect(refusal(removeRoad(linked, t, 5))).toContain('Link Road would end 17');
    // As a lane it may be left so (the Problems list says so).
    const lane = { ...linked, roads: roads.map((r, i) => (i === 4 ? { ...r, kind: 'lane' as const, area: 0 } : r)) };
    expect('draft' in removeRoad(lane, t, 5)).toBe(true);
  });

  test("the far stretch of a street drawn over the river takes its bridge with it (a bridge leads somewhere); the stretch before stays, a street if it led onto it out of town", () => {
    const drawn = made(draw(town(), [[87, MAIN_Z + 8], [87, 260]])).draft;
    const bridge = drawn.roads.findIndex((r) => r.bridge);
    const far = drawn.roads.findIndex((r, i) => i >= 4 && !r.bridge && r.ends.includes('dead'));
    expect(bridge).toBeGreaterThan(3);
    expect(far).toBeGreaterThan(3);
    const goes = whatGoes(drawn, t, far);
    if ('refused' in goes) throw new Error(goes.refused);
    expect(goes.roads).toEqual([bridge, far].sort((a, b) => a - b));
    expect(goes.bridges).toEqual([bridge]);
    expect(whatGoesLine(drawn, goes, far)).toEqual([`1 bridge left leading nowhere (${drawn.roads[bridge].name})`]);
    const after = made(removeRoad(drawn, t, far)).draft;
    expect(after.roads.some((r) => r.bridge)).toBe(false);
    expect(after.roads).toHaveLength(5);
    // The river is wet again where the deck was; the stretch from the high street ends by the bank.
    expect(unrle(after.open!.water, W * D)[215 * W + 87]).toBe(1);
    expect(after.roads[4].ends).toEqual(['join', 'dead']);
    expect(oneNetwork(after.roads)).toBe(true);
    // Out of the town's box: a road from Far Lane south 51 m onto a bridge, and on to Low Road past it. The country road
    // that led onto the bridge, left a long dead end, is made a street of no area, as a drawn one ending there is.
    const south = town();
    const lead = south.roads.length;
    const [b, tail] = [lead + 1, lead + 2];
    const across: NeighbourhoodDraft = {
      ...south,
      roads: [
        ...south.roads,
        { points: line(200, FAR_Z + 3.5, 200, 345), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Ford Lane' },
        { points: line(200, 345, 200, 380), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Ford Bridge', bridge: [5, 30] },
        { points: line(200, 380, 200, 390.5), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Ford Row' },
        { points: line(100, 394, 300, 394), width: 7, ends: ['dead', 'dead'], kind: 'country', name: 'Low Road' },
      ],
    };
    const taken = whatGoes(across, t, tail);
    if ('refused' in taken) throw new Error(taken.refused);
    expect(taken.bridges).toEqual([b]);
    expect(taken.streets).toEqual([lead]);
    const left = made(removeRoad(across, t, tail)).draft.roads.find((r) => r.name === across.roads[lead].name)!;
    expect(left).toMatchObject({ kind: 'street', ends: ['join', 'dead'] });
    expect(left.area).toBeUndefined();
  });

  test("refused if it would take the town's last homes (as a lot removed on its own is)", () => {
    // Only the far lane's homes: the road round the river's end takes them all with the far lane.
    const all = town();
    const farOnly: NeighbourhoodDraft = { ...all, slots: all.slots.slice(7).map((s, i) => ({ ...s, index: i })) };
    const goes = whatGoes(farOnly, t, 3);
    expect('refused' in goes && goes.refused).toContain('The town keeps its last home');
    expect(refusal(removeRoad(farOnly, t, 3))).toContain('The town keeps its last home');
    // With the high street's homes as well, it may go.
    expect('draft' in removeRoad(all, t, 3)).toBe(true);
  });

  test('its parking goes; a road joined to it ends there in a dead end; the water under a bridge is water again', () => {
    const goes = whatGoes(town(), t, 1);
    expect(goes).toEqual({ roads: [1], bridges: [], streets: [], lots: [], backdrop: [], parking: [0] });
    const { draft } = made(removeRoad(town(), t, 1));
    expect(draft.roads.length).toBe(3);
    expect(draft.open!.parking).toEqual([]);
    const bridged = made(draw(town(), [[120, MAIN_Z + 6], [120, FAR_Z - 6]])).draft;
    const bridge = bridged.roads.findIndex((r) => r.bridge);
    const after = made(removeRoad(bridged, t, bridge)).draft;
    // The street either side of it now ends at the water.
    expect(after.roads.filter((r) => r.ends.includes('dead')).length).toBe(bridged.roads.filter((r) => r.ends.includes('dead')).length + 2);
    // The river is wet again where the deck was.
    expect(unrle(after.open!.water, W * D)[215 * W + 120]).toBe(1);
    expect(oneNetwork(after.roads)).toBe(true);
  });
});

describe('renaming', () => {
  test("a road, its houses' addresses with it", () => {
    const { draft } = made(rename(town(), { kind: 'road', road: 0 }, '  Market   Street '));
    expect(draft.roads[0].name).toBe('Market Street');
    expect(draft.slots[0].name).toBe('1 Market Street');
    expect(draft.slots[7].name).toBe('7 Far Lane');
  });

  test('a name taken, or none, is refused', () => {
    expect(refusal(rename(town(), { kind: 'road', road: 0 }, 'Far Lane'))).toBe('Far Lane is taken');
    expect(refusal(rename(town(), { kind: 'road', road: 0 }, 'Ashby'))).toBe('Ashby is taken');
    expect(refusal(rename(town(), { kind: 'road', road: 0 }, '   '))).toBe('a name needs a letter or two');
  });

  test('the town is its main area', () => {
    const { draft } = made(rename(town(), { kind: 'town' }, 'Ashwell'));
    expect(draft.name).toBe('Ashwell');
    expect(draft.open!.areas[0].name).toBe('Ashwell');
  });
});

// A street drawn with `fill` on a generated world (not the hand-made town above, which has no lot
// mix): it gets homes from the world's lot mix (`lots.ts` `fillStreet`), along it and numbered on it.
describe('a street drawn with homes along it, on a generated world', async () => {
  const WEB = new URL('../', import.meta.url).pathname;
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { objectInfos } = await import('../src/game/housegen');
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { entriesOf } = await import('../src/game/edit/ground');
  const real = await Content.load('http://localhost/content/base.json');
  useTownContent(real);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const templates: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(real), lampTiles: real.lampTiles, windowTiles: real.windowTiles };
  const hamlet = generateWorld(real, templates, 'country', 4242, undefined, undefined, { town: 'hamlet', villages: 'none' });

  /** The first street that may be drawn off one of the main town's streets, square off it, `long` m (what is in its way cleared), that gets homes. */
  function firstStreet(draft: NeighbourhoodDraft, long: number): { points: Pt[]; drawn: NeighbourhoodDraft; cleared: number } {
    for (const r of draft.roads) {
      if (r.area !== 0 || r.kind === 'trail' || r.bridge) continue;
      const ls = lengths(r.points);
      for (let s = 10; s < ls[ls.length - 1] - 10; s += 12) {
        const { p, t: tq } = pointAt(r.points, s, ls);
        for (const side of [1, -1]) {
          const n: Pt = [-tq[1] * side, tq[0] * side];
          const from: Pt = [p[0] + n[0] * (r.width / 2 + 2), p[1] + n[1] * (r.width / 2 + 2)];
          const points: Pt[] = [from, [from[0] + n[0] * long, from[1] + n[1] * long]];
          const e = drawStreet(draft, templates, real, { points, kind: 'street', clear: true, fill: true, seed: 11 });
          if ('draft' in e && e.draft.roads.length > draft.roads.length && e.draft.slots.length > draft.slots.length - e.cleared.lots.length) return { points, drawn: e.draft, cleared: e.cleared.lots.length };
        }
      }
    }
    throw new Error('no street with homes could be drawn');
  }

  test('it gets homes from the lot mix, along it', () => {
    // (A dead end off a street, a court's length: the hamlet's streets leave no room for one joined at both ends.)
    const { points, drawn, cleared } = firstStreet(hamlet, 40);
    const road = drawn.roads.length - 1;
    const e = entriesOf(drawn, templates);
    const along = drawn.slots.filter((s, i) => s.kind === 'house' && e[i].road === road);
    expect(along.length).toBeGreaterThan(0);
    expect(drawn.slots.length).toBe(hamlet.slots.length - cleared + along.length);
    // Numbered on it, by its name.
    for (const s of along) expect(s.name.endsWith(` ${drawn.roads[road].name}`)).toBe(true);
    // The same stroke without homes has none along it.
    const bare = drawStreet(hamlet, templates, real, { points, kind: 'street', clear: true, fill: false, seed: 11 });
    if ('refused' in bare) throw new Error(bare.refused);
    expect(bare.draft.slots.length).toBe(hamlet.slots.length - cleared);
  });
});
