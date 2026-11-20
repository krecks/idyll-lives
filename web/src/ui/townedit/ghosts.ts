/**
 * The ghosts of step 2's edits (`TownGhosts`, drawn over the town by the renderer): what the plan
 * draft has that the town on screen hasn't yet, and what it no longer has, until the town is built
 * again with them (decision 10: an edit shows at once, the real town follows once edits pause).
 *
 * The two drafts are compared by what each thing is, not by its index (indices move as lots and
 * roads go): a lot by its rectangle, house and kind, a road by its line, a backdrop building by its
 * footprint and use. A lot or building the plan has and the town on screen hasn't is `new`; one it
 * no longer has is `gone` (its building dimmed). A road the plan has anew is `laid`, drawn as the road
 * it will be (near asphalt) so a street drawn shows as a road at once (`bridge` for a deck); one it no
 * longer has is drawn `bad`, red, as the street tool draws where one may not go: the renderer has no
 * dimmed road.
 */

import type { BackdropBuilding, RoadPlan } from '../../core/protocol';
import { plotRect, type NeighbourhoodDraft, type PlotSlot, type Templates } from '../../game/town';
import type { TownGhosts } from '../../render/types';

type Box = { x: number; z: number; w: number; d: number };
type LotGhost = TownGhosts['lots'][number];
type RoadGhost = TownGhosts['roads'][number];

/** A lot's rectangle on the town (m). */
export const lotBox = (t: Templates, s: PlotSlot): Box => plotRect(t, s);

/** A backdrop building's footprint's box (m): the ghost a lot outline can give it. */
export function buildingBox(b: Pick<BackdropBuilding, 'poly'>): Box {
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of b.poly) [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x), Math.max(z1, z)];
  return { x: x0, z: z0, w: x1 - x0, d: z1 - z0 };
}

/** A road as a ghost ribbon (a new or laid one with a deck as the bridge's own). */
export function roadGhost(r: RoadPlan, tone: RoadGhost['tone']): RoadGhost {
  return { points: r.points.map(([x, z]): [number, number] => [x, z]), width: r.width, tone: r.bridge && (tone === 'new' || tone === 'laid') ? 'bridge' : tone };
}

const lotSig = (t: Templates, s: PlotSlot) => {
  const r = plotRect(t, s);
  return `${r.x},${r.z},${r.w},${r.d}|${s.facing}|${s.kind}|${s.template}`;
};
const boxSig = (b: Box) => `${b.x},${b.z},${b.w},${b.d}`;
const roadSig = (r: RoadPlan) => `${r.width}|${r.bridge?.join(',') ?? ''}|${r.points.map(([x, z]) => `${x.toFixed(2)},${z.toFixed(2)}`).join(';')}`;
const buildingSig = (b: BackdropBuilding) => `${b.use}|${b.storeys}|${b.roof}|${b.poly.map(([x, z]) => `${x},${z}`).join(';')}`;

/**
 * What `plan` changed from `shown` (the town on screen), as ghosts: null if nothing it shows changed
 * (a rename, say). A lot whose rectangle stays but whose house or kind changed is one `new` outline
 * (not a `gone` under a `new`); the same for a building whose use changed.
 */
export function editGhosts(shown: NeighbourhoodDraft, plan: NeighbourhoodDraft, t: Templates): TownGhosts | null {
  if (shown === plan) return null;
  const lots: LotGhost[] = [];
  const roads: RoadGhost[] = [];
  if (shown.slots !== plan.slots) {
    const before = new Map(shown.slots.map((s) => [lotSig(t, s), s]));
    const after = new Map(plan.slots.map((s) => [lotSig(t, s), s]));
    const fresh = new Set<string>();
    for (const [sig, s] of after) {
      if (before.has(sig)) continue;
      const box = plotRect(t, s);
      fresh.add(boxSig(box));
      lots.push({ ...box, tone: 'new' });
    }
    for (const [sig, s] of before) {
      if (after.has(sig)) continue;
      const box = plotRect(t, s);
      if (!fresh.has(boxSig(box))) lots.push({ ...box, tone: 'gone' });
    }
  }
  if (shown.roads !== plan.roads) {
    const before = new Set(shown.roads.map(roadSig));
    const after = new Set(plan.roads.map(roadSig));
    for (const r of plan.roads) if (!before.has(roadSig(r))) roads.push(roadGhost(r, 'laid'));
    for (const r of shown.roads) if (!after.has(roadSig(r))) roads.push(roadGhost(r, 'bad'));
  }
  const was = shown.open?.backdrop ?? [];
  const now = plan.open?.backdrop ?? [];
  if (was !== now) {
    const before = new Set(was.map(buildingSig));
    const after = new Set(now.map(buildingSig));
    const fresh = new Set<string>();
    for (const b of now) {
      if (before.has(buildingSig(b))) continue;
      const box = buildingBox(b);
      fresh.add(boxSig(box));
      lots.push({ ...box, tone: 'new' });
    }
    for (const b of was) {
      if (after.has(buildingSig(b))) continue;
      const box = buildingBox(b);
      if (!fresh.has(boxSig(box))) lots.push({ ...box, tone: 'gone' });
    }
  }
  return lots.length || roads.length ? { roads, lots, boxes: [] } : null;
}

/** Ghosts drawn together: `over`'s after `under`'s (drawn last, on top). */
export function joinGhosts(under: TownGhosts | null, over: Partial<TownGhosts> | null): TownGhosts | null {
  if (!over) return under;
  const out = { roads: [...(under?.roads ?? []), ...(over.roads ?? [])], lots: [...(under?.lots ?? []), ...(over.lots ?? [])], boxes: [...(under?.boxes ?? []), ...(over.boxes ?? [])] };
  return out.roads.length || out.lots.length || out.boxes.length ? out : null;
}
