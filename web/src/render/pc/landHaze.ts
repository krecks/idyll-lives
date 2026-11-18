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
 * The curtain (`curtain`): the landscape's cut edge closed in the sky's colour behind it (as the veil, whole there,
 * takes the land beside it to), from over the ground there to under it and the sea's sheet. A camera past the edge (the step camera keeps its target over the land, not its eye) saw
 * the cut: the ground tiles' skirts standing as a green wall, and under the hills the sea's sheet the whole square has.
 * One draw of a few thousand triangles (about 2,700 round a medium land), whatever the camera's distance.
 *
 * The game's town view has both too, round the game's land (`hold`), only while the cut edge of its landscape is in
 * the picture (`shared/landVeil.ts` `cutInView`: from far out over a small land's corner, the eye past the landscape
 * looking back at it); nothing drawn or told otherwise.
 */

import * as pc from './engine';
import { curtain, VEIL_NEAR, veilAt, veilRect, type VeilRect } from '../shared/landVeil';
import { WATER_Y } from '../shared/nature';
import { landVeilTold, setLandVeil, skyFog } from './shaders/fog';

/** The camera's distance (m) the veil fades in over. */
const SHOW_FROM = 650;
const SHOW_FULL = 1050;

export class LandHaze {
  private readonly material: pc.StandardMaterial;
  private readonly entity: pc.Entity;
  private key = '';
  /** The veil's strength as shown (`show`, `hold`; -1: none), and whether the curtain is drawn with it at 0 too. */
  private shown = -1;
  private always = false;
  private rect: VeilRect | null = null;
  private readonly at = new pc.Vec3();
  /** The curtain's samples' tops, then their feet (game space; `set`, `shared/landVeil.ts` `curtain`). */
  positions: Float32Array = new Float32Array(0);

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
    mat.emissive.set(0, 0, 0);
    mat.cull = pc.CULLFACE_NONE;
    // (Its colour the sky's dome behind it, as the veil takes the land beside it to: `shaders/fog.ts` `skyFog`.)
    mat.shaderChunksVersion = '2.8';
    mat.getShaderChunks(pc.SHADERLANGUAGE_GLSL).set('fogPS', skyFog('glsl'));
    mat.getShaderChunks(pc.SHADERLANGUAGE_WGSL).set('fogPS', skyFog('wgsl'));
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
    const { positions, indices } = curtain(box, height, WATER_Y);
    this.positions = positions;
    // (The old mesh goes with its mesh instance.)
    if (this.entity.render) this.entity.removeComponent('render');
    const mesh = new pc.Mesh(this.app.graphicsDevice);
    mesh.setPositions(positions);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const mi = new pc.MeshInstance(mesh, this.material);
    this.entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false, layers: [layer] });
    this.entity.enabled = this.shown > 0 || (this.shown === 0 && this.always);
  }

  /**
   * How far out the camera is (m), and where its eye is (`ex`, `ez`, game space): the veil fades in from `SHOW_FROM`,
   * and as the eye goes out over the landscape's last `VEIL_FADE` m, as thick as the veil is under it (a camera past
   * the land's edge is in the haze, and sees the land's cut edge go into it); the curtain is drawn. False: neither
   * (the game's view). Told to the shaders only when it, or where the land stands, changed. Allocation-free.
   */
  show(radius: number | false, ex = NaN, ez = NaN): void {
    if (radius === false) return this.hold(false);
    const r = this.rect;
    const t = Math.min(1, Math.max(0, (radius - SHOW_FROM) / (SHOW_FULL - SHOW_FROM)));
    this.hold(Math.max(t * t * (3 - 2 * t), r && Number.isFinite(ex) && Number.isFinite(ez) ? veilAt(r, ex, ez) : 0), true);
  }

  /**
   * The veil at `k` (0..1) and the curtain drawn with it (`curtain`: always, as the menus' town has it; else while
   * `k` is over 0, as the game's has it), or neither (false). Told to the shaders only when it, or where the land
   * stands, changed from what they have (`landVeilTold`: another's, the menus' or the game's, may have told them
   * since). Allocation-free.
   */
  hold(k: number | false, curtain = false): void {
    const r = this.rect;
    this.shown = k === false ? -1 : k;
    this.always = curtain;
    this.entity.enabled = k !== false && (curtain || k > 0) && !!this.entity.render;
    const strength = r ? Math.max(0, this.shown) : 0;
    // (The veil's middle in engine space: the land's root is mirrored in z, and lowered by the datum.)
    const p = r ? this.parent.getWorldTransform().transformPoint(this.at.set(r.cx, 0, r.cz), this.at) : this.at.set(0, 0, 0);
    const told = landVeilTold(this.app.graphicsDevice);
    // (As the shaders keep them, in single precision; its ends exactly: clear is none, whole is whole.)
    const k32 = Math.fround(strength);
    if (told && Math.fround(p.x) === told.cx && Math.fround(p.z) === told.cz && Math.fround(r?.hx ?? 0) === told.hx && Math.fround(r?.hz ?? 0) === told.hz && (k32 === told.strength || (Math.abs(strength - told.strength) < 0.002 && strength > 0 && strength < 1))) return;
    setLandVeil(this.app.graphicsDevice, p.x, p.z, r?.hx ?? 0, r?.hz ?? 0, r?.fade ?? 1, strength, VEIL_NEAR);
  }

  destroy(): void {
    setLandVeil(this.app.graphicsDevice, 0, 0, 0, 0, 1, 0, 1);
    this.entity.destroy();
    this.material.destroy();
  }
}
