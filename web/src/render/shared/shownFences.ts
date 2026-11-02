/**
 * The fences the renderer draws: the world's own (`WorldStructure.fences`), and in a town from before
 * real garden fences (no `WorldMeta.gardenFences`: older saves) the ones its town would have put up
 * (`game/gardenFences.ts`), planned from what the save has, drawn only. They stand where a new town's
 * would, whichever lot is in view, none on the player's lot (the player builds there), none where a
 * wall or a fence of the player's already is; residents walk through them as they always did there.
 */

import type { FenceStyleDef } from '../../content/content';
import { groundDepth, type FenceEdge, type WorldStructure } from '../../core/protocol';
import { townFences, type FencePlot } from '../../game/gardenFences';
import type { Facing, Rect } from '../../game/town';

/** Driveways are paths at least this wide (`game/town.ts` `DRIVEWAY_MIN_WIDTH`). */
const DRIVEWAY = 1.8;

const memo = new WeakMap<WorldStructure, { refs: unknown[]; fences: FenceEdge[] }>();
let last: { refs: unknown[]; fences: FenceEdge[] } | null = null;

/** The fences to draw for `world` (the same array while what they come from stays the same). */
export function shownFences(world: WorldStructure, styles: readonly FenceStyleDef[]): FenceEdge[] {
  const m = world.meta;
  if (!m || m.gardenFences || !m.paths) return world.fences ?? NONE;
  const player = world.households.find((h) => h.player)?.plot ?? undefined;
  const refs = [m, world.walls, world.objects, world.plots, world.garages, world.fences, player, styles];
  const known = memo.get(world) ?? last;
  if (known && known.refs.every((r, i) => r === refs[i])) return known.fences;
  const fences = [...(world.fences ?? []), ...planned(world, styles, player)];
  last = { refs, fences };
  memo.set(world, last);
  return fences;
}
const NONE: FenceEdge[] = [];

function planned(world: WorldStructure, styles: readonly FenceStyleDef[], player: number | undefined): FenceEdge[] {
  const m = world.meta!;
  const D = groundDepth(world);
  const paths = m.paths ?? [];
  const streets = (m.streets ?? []).filter((s) => s.w >= s.d);
  const inside = (r: Rect, p: Rect) => r.x >= p.x - 0.01 && r.x + r.w <= p.x + p.w + 0.01 && r.z >= p.z - 0.01 && r.z + r.d <= p.z + p.d + 0.01;
  const plots: FencePlot[] = [];
  world.plots.forEach((p, i) => {
    // Its street's side: as the town says, else (older saves) the street one of its z edges lies on (as the street lays it out).
    let facing = m.facing?.[i] as Facing | undefined;
    if (facing === undefined) {
      for (const st of streets) {
        if (p.x + p.w <= st.x || p.x >= st.x + st.w) continue;
        if (Math.abs(p.z + p.d - st.z) < 0.05) facing = 2;
        else if (Math.abs(p.z - (st.z + st.d)) < 0.05) facing = 0;
      }
    }
    if (facing === undefined) return;
    const drive = paths.find((r) => inside(r, p) && Math.min(r.w, r.d) >= DRIVEWAY) ?? null;
    const g = world.garages?.find((g) => g.plot === p.id);
    plots.push({
      rect: { x: p.x, z: p.z, w: p.w, d: p.d },
      facing,
      house: p.house,
      home: !p.public,
      drive,
      garage: g ? { x: g.x0, z: g.z0, w: g.x1 - g.x0, d: g.z1 - g.z0 } : null,
      fenced: p.id !== player,
    });
  });
  // No fence on a wall or on one already there (the player's), nor through a thing.
  const walls = new Set<string>();
  for (const e of world.walls ?? []) if (e.z <= D) walls.add(`${e.axis}:${e.x}:${e.z}`);
  for (const f of world.fences ?? []) walls.add(`${f.axis}:${f.x}:${f.z}`);
  const blockers = world.objects.filter((o) => o.z < D).map((o) => ({ x: o.x, z: o.z, w: o.w, d: o.d }));
  const style = (id: string) => Math.max(0, styles.findIndex((s) => s.id === id));
  return townFences({ plots, fields: m.world?.fields ?? [], walls, blockers, paths }).map((f) => ({ axis: f.axis, x: f.x, z: f.z, kind: f.kind, style: style(f.style) }));
}
