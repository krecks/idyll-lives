/**
 * A body measured in its clips, once as it loads (`residents.ts`): where the seated clips put the hips, where the
 * legs are in the clips whose feet are placed, the lying pose a quilt is draped over, and the heights clothes are
 * cut at. Skinned on the CPU with the residents' own posing (`pose.ts`), engine-neutral.
 */

import type { BodyData, CharacterSet } from './data';
import type { OutfitFit } from './outfit';
import { resetAdditive, sampleClip, skin, type Additive, type PoseScratch } from './pose';
import { bindY } from './rig';
import { LEG_CLIPS, LIE_BACK, LIE_CENTRE, type LegFit, type SeatFit } from './seats';

/** The residents' posing scratch a measurement poses a body with (`Characters`' own; the poses it leaves there are its). */
export interface PoseKit {
  set: CharacterSet;
  qA: Float32Array;
  pA: Float32Array;
  add: Additive;
  scratch: PoseScratch;
}

/** A base body's heights for the towel, swimwear and an open jacket's shirt (`OutfitFit`), from its joints at rest. */
export function outfitFit(set: CharacterSet, base: BodyData): OutfitFit {
  const y = (bone: string) => bindY(base.inverseBind, set.bone[bone]);
  return { chest: (y('spine_03') + y('upperarm_l')) / 2, hip: y('thigh_l'), knee: y('calf_l'), neck: y('neck_01') - 0.01 };
}

/**
 * The lying pose of a body (scale 1, slot frame: x across, y above the mattress, z towards the
 * feet), skinned on the CPU once for the blanket: the `lie` clip sleeping Sims play. Also returns
 * the chest line and the toes' z.
 */
export function lyingPose(kit: PoseKit, body: BodyData): {
  positions: Float32Array;
  chestZ: number;
  feetZ: number;
} {
  const set = kit.set;
  const B = set.bone;
  const NB = set.bones.length;
  const mats = new Float32Array(NB * 16);
  const head = new Float32Array(3);
  sampleClip(set, set.clips.get('lie') ?? set.clips.get('idle')!, 0, kit.qA, kit.pA);
  resetAdditive(kit.add);
  const sp = Math.sin(-Math.PI / 4);
  const cp = Math.cos(-Math.PI / 4);
  skin(
    set,
    body,
    kit.qA,
    kit.pA,
    kit.add,
    {
      x: 0,
      // (As `lieBack`: a smaller body than its family's base body lies lower.)
      y: LIE_BACK * (body.height / (set.bodies.get(body.family)?.height ?? body.height)),
      z: LIE_CENTRE,
      qx: sp,
      qy: 0,
      qz: 0,
      qw: cp,
      scale: 1,
    },
    kit.scratch,
    mats,
    0,
    head,
  );
  const chestZ = Math.max(kit.scratch.worldT[B.upperarm_l * 3 + 2], kit.scratch.worldT[B.upperarm_r * 3 + 2]) + 0.07;
  resetAdditive(kit.add);
  // The body plus the clothes that stand off it most (skirts, boots).
  const parts = [...Object.keys(body.bodyChunks), 'bottom.skirt', 'shoes.boots'].map((k) => body.parts.get(k)).filter((p): p is NonNullable<typeof p> => !!p);
  const total = parts.reduce((n, p) => n + p.positions.length / 3, 0);
  const out = new Float32Array(total * 3);
  let feetZ = -Infinity;
  let base = 0;
  for (const part of parts) {
    const n = part.positions.length / 3;
    for (let v = 0; v < n; v++) {
      const px = part.positions[v * 3];
      const py = part.positions[v * 3 + 1];
      const pz = part.positions[v * 3 + 2];
      let x = 0;
      let y = 0;
      let z = 0;
      for (let k = 0; k < 4; k++) {
        const w = part.weights[v * 4 + k];
        if (w <= 0) continue;
        const m = part.joints[v * 4 + k] * 16;
        x += w * (mats[m] * px + mats[m + 4] * py + mats[m + 8] * pz + mats[m + 12]);
        y += w * (mats[m + 1] * px + mats[m + 5] * py + mats[m + 9] * pz + mats[m + 13]);
        z += w * (mats[m + 2] * px + mats[m + 6] * py + mats[m + 10] * pz + mats[m + 14]);
      }
      out[(base + v) * 3] = x;
      out[(base + v) * 3 + 1] = y;
      out[(base + v) * 3 + 2] = z;
      if (part === parts[0] && z > feetZ) feetZ = z;
    }
    base += n;
  }
  return { positions: out, chestZ, feetZ };
}

/** Where each leg is in the clips whose feet are placed (`LEG_CLIPS`), at their start (body space, scale 1). */
export function measureLegs(kit: PoseKit, body: BodyData): Map<string, LegFit> {
  const set = kit.set;
  const B = set.bone;
  const tmp = new Float32Array(set.bones.length * 16);
  const head = new Float32Array(3);
  const t = kit.scratch.worldT;
  const fits = new Map<string, LegFit>();
  // The ankles' height standing (the idle's first frame), for feet on the floor.
  sampleClip(set, set.clips.get('idle')!, 0, kit.qA, kit.pA);
  resetAdditive(kit.add);
  skin(set, body, kit.qA, kit.pA, kit.add, { x: 0, y: 0, z: 0, qx: 0, qy: 0, qz: 0, qw: 1, scale: 1 }, kit.scratch, tmp, 0, head);
  const ankle = t[B.foot_l * 3 + 1];
  for (const key of LEG_CLIPS) {
    const clip = set.clips.get(key);
    if (!clip) continue;
    sampleClip(set, clip, 0, kit.qA, kit.pA);
    resetAdditive(kit.add);
    skin(set, body, kit.qA, kit.pA, kit.add, { x: 0, y: 0, z: 0, qx: 0, qy: 0, qz: 0, qw: 1, scale: 1 }, kit.scratch, tmp, 0, head);
    const side = (thigh: number, calf: number, foot: number) => {
      const [hy, hz, ky, kz, ay, az] = [t[thigh * 3 + 1], t[thigh * 3 + 2], t[calf * 3 + 1], t[calf * 3 + 2], t[foot * 3 + 1], t[foot * 3 + 2]];
      return { hy, hz, l1: Math.hypot(ky - hy, kz - hz), l2: Math.hypot(ay - ky, az - kz), t0: Math.atan2(kz - hz, hy - ky), c0: Math.atan2(az - kz, ky - ay) };
    };
    fits.set(key, {
      l: side(B.thigh_l, B.calf_l, B.foot_l),
      r: side(B.thigh_r, B.calf_r, B.foot_r),
      spine: [t[B.spine_01 * 3 + 1], t[B.spine_01 * 3 + 2]],
      head: [t[B.Head * 3 + 1], t[B.Head * 3 + 2]],
      ankle,
    });
  }
  return fits;
}

/** Pelvis height / forward offset of the seated and lying poses (body space, scale 1): `sit`'s, or `key`'s (`sitFloor`). */
export function measureSit(kit: PoseKit, body: BodyData, key = 'sit'): SeatFit {
  const set = kit.set;
  const tmp = new Float32Array(set.bones.length * 16);
  const head = new Float32Array(3);
  const pose = (key: string, t: number) => {
    const clip = set.clips.get(key)!;
    sampleClip(set, clip, t, kit.qA, kit.pA);
    resetAdditive(kit.add);
    skin(set, body, kit.qA, kit.pA, kit.add, { x: 0, y: 0, z: 0, qx: 0, qy: 0, qz: 0, qw: 1, scale: 1 }, kit.scratch, tmp, 0, head);
    const p = set.bone.pelvis * 3;
    return { y: kit.scratch.worldT[p + 1], z: kit.scratch.worldT[p + 2] };
  };
  const sit = pose(key, 0);
  // The pelvis joint rests `seatDrop` above the seat surface (sets without it: ~11 cm).
  return { y: sit.y - (body.seatDrop ?? 0.11), z: sit.z };
}
