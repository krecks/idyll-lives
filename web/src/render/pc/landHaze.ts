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
 *
 * The curtain (`curtain`): the landscape's cut edge closed in the haze's colour, from over the ground there to under
 * it and the sea's sheet. A camera past the edge (the step camera keeps its target over the land, not its eye) saw
 * the cut: the ground tiles' skirts standing as a green wall, and under the hills the sea's sheet the whole square has.
 * One draw of a few thousand triangles (about 2,700 round a medium land), whatever the camera's distance.
 */

import * as pc from './engine';
import { curtain, VEIL_NEAR, veilAt, veilRect, type VeilRect } from '../shared/landVeil';
import { WATER_Y } from '../shared/nature';
import { setLandVeil } from './shaders/fog';

/** The camera's distance (m) the veil fades in over. */
const SHOW_FROM = 650;
const SHOW_FULL = 1050;

export class LandHaze {
  private readonly material: pc.StandardMaterial;
  private readonly entity: pc.Entity;
  private key = '';
  /** The veil's strength as shown (`show`), and where it was told to the shaders from (engine space). */
  private shown = -1;
  private rect: VeilRect | null = null;
  private readonly told = { x: NaN, z: NaN, k: -1 };
  private readonly at = new pc.Vec3();

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
  ) {
    const mat = (this.material = new pc.StandardMaterial());
    mat.name = 'landHaze';
    mat.useLighting = false;
    mat.useSkybox = false;
    mat.useFog = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.cull = pc.CULLFACE_NONE;
    mat.update();
    this.entity = new pc.Entity('landHaze', app);
    this.entity.enabled = false;
    parent.addChild(this.entity);
  }

  /**
   * The veil and the curtain round the landscape `box` (m: from x0, z0 to x1, z1, its ring included) over the land
   * `height` has, on `layer`. Made again only for another box or another land (`key`).
   */
  set(box: { x0: number; z0: number; x1: number; z1: number }, height: (x: number, z: number) => number, layer: number, key: string): void {
    const full = `${box.x0},${box.z0},${box.x1},${box.z1}|${key}|${layer}`;
    if (full === this.key) return;
    this.key = full;
    this.rect = veilRect(box);
    this.told.k = -1;
    const { positions, indices } = curtain(box, height, WATER_Y);
    // (The old mesh goes with its mesh instance.)
    if (this.entity.render) this.entity.removeComponent('render');
    const mesh = new pc.Mesh(this.app.graphicsDevice);
    mesh.setPositions(positions);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const mi = new pc.MeshInstance(mesh, this.material);
    this.entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false, layers: [layer] });
    this.entity.enabled = this.shown >= 0;
  }

  /** Its colour: the haze's (`scene.fog.color`, as PlayCanvas takes a colour: the sky's dome comes out the same). */
  setColour(c: pc.Color): void {
    this.material.emissive.set(c.r, c.g, c.b);
    this.material.update();
  }

  /**
   * How far out the camera is (m), and where its eye is (`ex`, `ez`, game space): the veil fades in from `SHOW_FROM`,
   * and as the eye goes out over the landscape's last `VEIL_FADE` m, as thick as the veil is under it (a camera past
   * the land's edge is in the haze, and sees the land's cut edge go into it); the curtain is drawn. False: neither
   * (the game's view). Told to the shaders only when it, or where the land stands, changed. Allocation-free.
   */
  show(radius: number | false, ex = NaN, ez = NaN): void {
    const r = this.rect;
    const t = radius === false ? 0 : Math.min(1, Math.max(0, (radius - SHOW_FROM) / (SHOW_FULL - SHOW_FROM)));
    const k = Math.max(t * t * (3 - 2 * t), r && Number.isFinite(ex) && Number.isFinite(ez) ? veilAt(r, ex, ez) : 0);
    this.shown = radius === false ? -1 : k;
    this.entity.enabled = radius !== false && !!this.entity.render;
    const strength = r ? Math.max(0, this.shown) : 0;
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
    this.entity.destroy();
    this.material.destroy();
  }
}
