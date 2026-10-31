/**
 * A baby in someone's arms (`CUDDLE`, `CARRY`, `CHANGE` in `actions.ts`; `residents.ts` `cuddles` works out who holds
 * whom and how far along they are): the grown-up's pose over the crib and holding the baby, and where the baby is
 * placed: in the hands, cradled on the forearms, or on the changing pad.
 */

import { BEDS, type PlacedObject } from './seats';
import { CHANGE, CRADLE, CUDDLE } from './actions';
import { addRotation, qmul, rotate, type Additive, type Placement } from './pose';
import { clamp, lerp, lieBack, smooth, type SimRig } from './rig';

/**
 * A baby on the crib's changing pad (`CHANGE`): their placement as in the cot (as posed), moved to the
 * pad beside it (across the crib's middle from the cot, a little back, on the pad's top) and turned
 * across it, about the body's middle.
 */
export function onPad(pl: Placement, q: Float32Array, rig: SimRig, crib: PlacedObject): void {
  const s = rig.scale;
  const my = rig.body.height * 0.5 * s;
  const mz = -lieBack(rig);
  rotate(pl.qx, pl.qy, pl.qz, pl.qw, 0, my, mz, q, 8);
  const x0 = pl.x + q[8];
  const y0 = pl.y + q[9];
  const z0 = pl.z + q[10];
  // The crib's right (+x) and front (+z), as sim-core lays its slots out.
  const f = (crib.rot * Math.PI) / 2;
  const rx = Math.cos(f);
  const rz = -Math.sin(f);
  const across = (x0 - crib.cx) * rx + (z0 - crib.cz) * rz;
  const x = crib.cx - across * rx + CHANGE.back * Math.sin(f);
  const z = crib.cz - across * rz + CHANGE.back * Math.cos(f);
  const y = y0 + CHANGE.pad - (BEDS.crib?.h ?? 0.56);
  qmul(0, Math.sin(CHANGE.turn / 2), 0, Math.cos(CHANGE.turn / 2), pl.qx, pl.qy, pl.qz, pl.qw, q, 0);
  [pl.qx, pl.qy, pl.qz, pl.qw] = [q[0], q[1], q[2], q[3]];
  rotate(pl.qx, pl.qy, pl.qz, pl.qw, 0, my, mz, q, 8);
  pl.x = x - q[8];
  pl.y = y - q[9];
  pl.z = z - q[10];
}

/** Where Sim row `i` (picking up or holding a baby) has the baby, as just posed: in their hands, or cradled on their forearms. */
export function writeCradle(B: Record<string, number>, t: Float32Array, c: Float32Array, i: number, rig: SimRig): void {
  // Reaching into the crib: between the hands; holding: half way along the forearms, a little forward.
  const h = smooth(rig.holdW);
  for (let a = 0; a < 3; a++) {
    const hands = (t[B.hand_l * 3 + a] + t[B.hand_r * 3 + a]) / 2;
    c[i * 3 + a] = lerp(hands, (hands + (t[B.lowerarm_l * 3 + a] + t[B.lowerarm_r * 3 + a]) / 2) / 2, h);
  }
  c[i * 3] += Math.sin(rig.yaw) * CRADLE.forward * rig.scale * h;
  c[i * 3 + 2] += Math.cos(rig.yaw) * CRADLE.forward * rig.scale * h;
}

/**
 * A baby in the arms of Sim row `h` (`CUDDLE`), `e` of the way (0..1) from lying in the crib (the
 * placement as posed) to cradled across their forearms, on the back with the head in the crook of
 * the left arm, raised a little and turned towards them. Moves the body's middle (half way up its
 * back) with the hands and turns it about that.
 */
export function cradled(pl: Placement, q: Float32Array, c: Float32Array, rig: SimRig, holder: SimRig, h: number, e: number): void {
  const s = rig.scale;
  const my = rig.body.height * 0.5 * s;
  const mz = -lieBack(rig);
  // Lying in the crib: the middle.
  rotate(pl.qx, pl.qy, pl.qz, pl.qw, 0, my, mz, q, 8);
  const x0 = pl.x + q[8];
  const y0 = pl.y + q[9];
  const z0 = pl.z + q[10];
  // In the arms: head towards the holder's left, along their front.
  const yaw = Math.atan2(Math.cos(holder.yaw), -Math.sin(holder.yaw));
  const pitch = -Math.PI / 2 + CRADLE.tilt;
  qmul(0, Math.sin(yaw / 2), 0, Math.cos(yaw / 2), Math.sin(pitch / 2), 0, 0, Math.cos(pitch / 2), q, 0);
  qmul(q[0], q[1], q[2], q[3], 0, Math.sin(CRADLE.roll / 2), 0, Math.cos(CRADLE.roll / 2), q, 4);
  // Blended with the crib's (the shorter way round).
  const dot = pl.qx * q[4] + pl.qy * q[5] + pl.qz * q[6] + pl.qw * q[7];
  const sg = dot < 0 ? -1 : 1;
  let qx = lerp(pl.qx, sg * q[4], e);
  let qy = lerp(pl.qy, sg * q[5], e);
  let qz = lerp(pl.qz, sg * q[6], e);
  let qw = lerp(pl.qw, sg * q[7], e);
  const len = Math.hypot(qx, qy, qz, qw) || 1;
  qx /= len;
  qy /= len;
  qz /= len;
  qw /= len;
  // (In the hands soon after they start lifting, turned round over the whole lift; cut short, back
  // over the rail into the crib.)
  const at = smooth(clamp(e * 2.2, 0, 1));
  const x = lerp(x0, c[h * 3], at);
  const y = lerp(y0, c[h * 3 + 1] + CRADLE.lift * s, at) + Math.sin(at * Math.PI) * (holder.cuddleT < 0 ? 0.2 : 0.04);
  const z = lerp(z0, c[h * 3 + 2], at);
  rotate(qx, qy, qz, qw, 0, my, mz, q, 8);
  pl.x = x - q[8];
  pl.y = y - q[9];
  pl.z = z - q[10];
  pl.qx = qx;
  pl.qy = qy;
  pl.qz = qz;
  pl.qw = qw;
}

/**
 * The grown-up's side of a cuddle (`CUDDLE`), by its weights: bent over the crib's rail with the
 * arms reaching down into it; and holding the baby cradled in the arms, looking down at them,
 * rocking from side to side.
 */
export function cuddlePose(add: Additive, B: Record<string, number>, rig: SimRig, time: number): void {
  const r = smooth(rig.reachW);
  const h = smooth(rig.holdW);
  if (r > 0.001) {
    // Bent over the rail (the taller, the further), the arms down into the crib, a little in.
    const lean = CUDDLE.lean * clamp(1 + (rig.body.height * rig.scale - CUDDLE.height) * 1.2, 0.75, 1.4);
    addRotation(add, B.spine_01, 1, 0, 0, lean * 0.67 * r);
    addRotation(add, B.spine_02, 1, 0, 0, lean * 0.33 * r);
    addRotation(add, B.upperarm_l, 0, 0, 1, 0.1 * r);
    addRotation(add, B.upperarm_r, 0, 0, 1, -0.1 * r);
    addRotation(add, B.upperarm_l, 1, 0, 0, -(lean + CUDDLE.arms) * r);
    addRotation(add, B.upperarm_r, 1, 0, 0, -(lean + CUDDLE.arms) * r);
    addRotation(add, B.lowerarm_l, 1, 0, 0, -0.15 * r);
    addRotation(add, B.lowerarm_r, 1, 0, 0, -0.15 * r);
    addRotation(add, B.Head, 1, 0, 0, 0.2 * r);
  }
  if (h > 0.001) {
    // Rocking: slowly from side to side, the shoulders turning with it.
    const sway = Math.sin(time * 2.1 + rig.seed * 6) * h;
    addRotation(add, B.spine_01, 0, 0, 1, 0.035 * sway);
    addRotation(add, B.spine_02, 0, 1, 0, 0.14 * sway);
    // Cradling: elbows bent, the forearms across the front.
    addRotation(add, B.upperarm_l, 0, 0, 1, -0.15 * h);
    addRotation(add, B.upperarm_r, 0, 0, 1, 0.15 * h);
    addRotation(add, B.upperarm_l, 1, 0, 0, -0.15 * h);
    addRotation(add, B.upperarm_r, 1, 0, 0, -0.15 * h);
    addRotation(add, B.lowerarm_l, 1, 0, 0, -1.0 * h);
    addRotation(add, B.lowerarm_r, 1, 0, 0, -1.0 * h);
    addRotation(add, B.lowerarm_l, 0, 1, 0, 1.1 * h);
    addRotation(add, B.lowerarm_r, 0, 1, 0, -1.1 * h);
    // Feeding from a bottle: the right forearm up off the legs, the bottle in the hand over the baby (the teat towards
    // their head, not quite at the mouth: the arms are posed, not reached with IK).
    if (rig.feeding) {
      addRotation(add, B.upperarm_r, 1, 0, 0, -0.35 * h);
      addRotation(add, B.lowerarm_r, 1, 0, 0, -0.4 * h);
      addRotation(add, B.lowerarm_r, 0, 1, 0, -0.5 * h);
    }
    // Looking down at the baby (the head on the left).
    addRotation(add, B.neck_01, 1, 0, 0, 0.15 * h);
    addRotation(add, B.Head, 1, 0, 0, 0.3 * h);
    addRotation(add, B.Head, 0, 1, 0, -0.2 * h);
  }
}
