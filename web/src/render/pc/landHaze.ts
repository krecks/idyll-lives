/**
 * The haze the menus' land fades into at its edge, from far out (the whole land's shot of the new game's steps).
 * The landscape reaches `landscapeRing` m past the land and no further; from 1,300 m up the picture took in the bare
 * sky's colour past it, a straight edge down one side. Drawing the landscape further would cost the 8 GB machines
 * its tiles, woods and grass over twice the ground.
 *
 * The veil (`shared/landVeil.ts`): the haze laid over the landscape's last `VEIL_FADE` m by where each thing stands,
 * clear at its inner edge and whole by the landscape's own, in the game's materials' own haze (`shaders/fog.ts`
 * `setLandVeil`): the land goes into the haze, and past it there is only the haze, the same colour as the sky's dome
 * behind it. It shows from `SHOW_FROM` m out (`show`): nearer, the play camera's and the area shots' haze is their own.
 * It is clear at the eye, whole from `VEIL_NEAR` m off (a camera out in it sees what is round it as it is).
 * (A sheet hung over the land and moved along the eye's lines to the land it was for came apart from a low camera:
 * pale slabs across the picture, over the hills and the sky.)
 */

import * as pc from './engine';
import { VEIL_NEAR, veilAt, veilRect, type VeilRect } from '../shared/landVeil';
import { setLandVeil } from './shaders/fog';

/** The camera's distance (m) the veil fades in over. */
const SHOW_FROM = 650;
const SHOW_FULL = 1050;

export class LandHaze {
  private key = '';
  /** The veil's rectangle, and its strength and middle (engine space) as told to the shaders (`show`). */
  private rect: VeilRect | null = null;
  private readonly told = { x: NaN, z: NaN, k: -1 };
  private readonly at = new pc.Vec3();

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
  ) {}

  /** The veil round the landscape `box` (m: from x0, z0 to x1, z1, its ring included). Made again only for another box (`key`). */
  set(box: { x0: number; z0: number; x1: number; z1: number }, key: string): void {
    const full = `${box.x0},${box.z0},${box.x1},${box.z1}|${key}`;
    if (full === this.key) return;
    this.key = full;
    this.rect = veilRect(box);
    this.told.k = -1;
  }

  /**
   * How far out the camera is (m), and where its eye is (`ex`, `ez`, game space): the veil fades in from `SHOW_FROM`,
   * and as the eye goes out over the landscape's last `VEIL_FADE` m, as thick as the veil is under it (a camera past
   * the land's edge is in the haze, and sees the land's edge go into it). False: none (the game's view). Told to the shaders only when it, or where the land stands, changed. Allocation-free.
   */
  show(radius: number | false, ex = NaN, ez = NaN): void {
    const r = this.rect;
    const t = radius === false ? 0 : Math.min(1, Math.max(0, (radius - SHOW_FROM) / (SHOW_FULL - SHOW_FROM)));
    const k = Math.max(t * t * (3 - 2 * t), r && Number.isFinite(ex) && Number.isFinite(ez) ? veilAt(r, ex, ez) : 0);
    const strength = r && radius !== false ? k : 0;
    // (The veil's middle in engine space: the land's root is mirrored in z, and lowered by the datum.)
    const p = r ? this.parent.getWorldTransform().transformPoint(this.at.set(r.cx, 0, r.cz), this.at) : this.at.set(0, 0, 0);
    const told = this.told;
    // (Its ends exactly: clear is none, whole is whole.)
    const near = strength === told.k || (Math.abs(strength - told.k) < 0.002 && strength > 0 && strength < 1);
    if (near && p.x === told.x && p.z === told.z) return;
    told.k = strength;
    told.x = p.x;
    told.z = p.z;
    setLandVeil(this.app.graphicsDevice, p.x, p.z, r?.hx ?? 0, r?.hz ?? 0, r?.fade ?? 1, strength, VEIL_NEAR);
  }

  destroy(): void {
    setLandVeil(this.app.graphicsDevice, 0, 0, 0, 0, 1, 0, 1);
  }
}
