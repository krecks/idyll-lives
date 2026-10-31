/**
 * Buy mode's thing in hand (`Renderer.setPlacementGhost`): the model itself where it fits, a red
 * see-through copy where it can't go (pulsing), and a flat arrow on the tile a resident uses it
 * from, pointing out of it (green, or red), riding with it but turned by quarter turns only. It
 * glides to its tile and turns the short way round, floats a hand's breadth above the floor (or
 * the top it goes on), bobbing gently, and springs in when a new thing is taken in hand. (Reduce
 * motion: held still, no spring, no pulse.)
 *
 * Its own copy of the model (`buildModel`, once per model, size and colours; without its file the
 * box the placed thing would get, at once from then on until the file comes: `fileArrived`), one
 * instance per mesh (and the arrow), in lot rows under the storey layer; on the over layer
 * (`layers.ts`: drawn after the world, the depth cleared, over what stands in front of it), casting
 * and taking no shadow (the thing itself lit by the world's lights). Moving it costs a matrix per
 * draw each frame: no allocations, no shader compiles (the red copy's alpha and the arrow's colour
 * are uniforms written in place).
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { overLayer } from './layers';
import { buildModel, disposeTemplate, type ModelTemplate } from './models';
import { ARROW_PS, GHOST_PS, GHOST_VS } from './shaders/ghost';
import { colorsKey, objectCentre, type PartColors } from '../../core/protocol';
import type { Lighting } from '../shared/environment';
import { compose, multiply, quatYawPitchRoll, toLinear, type Quat } from '../shared/math';
import { painted, standInOf } from '../shared/modelEntry';
import { resizeKey, type ObjectResize, type PlacementGhost, type RendererDeps } from '../types';

/** Buy mode's ghost's key: its model, kind (the box it gets without its file), size and colours. */
function ghostKeyOf(model: string, kind: string | undefined, resize?: ObjectResize, colors?: PartColors): string {
  return `${model}|${kind ?? ''}${resizeKey(resize)}${colorsKey(colors)}`;
}

/** `out` = (`x`, `y`, `z`) times `k`. */
function setScaled(out: Float32Array, x: number, y: number, z: number, k: number): void {
  out[0] = x * k;
  out[1] = y * k;
  out[2] = z * k;
}

/** Whether a mesh has vertex colours (the red copy multiplies by them, `GAME_GHOST_COLORS`). */
function hasColors(mesh: pc.Mesh): boolean {
  return !!mesh.vertexBuffer?.format.elements.some((e) => e.name === pc.SEMANTIC_COLOR);
}

/** Where each moving value is in `PlacementGhostView.now`. */
const AGE = 0;
const SPAWN = 1;
const X = 2;
const Y = 3;
const Z = 4;
const YAW = 5;
const ALPHA = 6;

type GhostArgs = [model: string, kind: string | undefined, resize: ObjectResize | undefined, colors: PartColors | undefined];

export class PlacementGhostView {
  private readonly device: pc.GraphicsDevice;
  private readonly layer: number;
  private key = '';
  /** Models whose file failed to load for the ghost, with the files: drawn as its box straight away from then on, until one comes (`fileArrived`). */
  private readonly unfiled = new Map<string, string[]>();
  /** What the ghost was last built from (`load`'s arguments). */
  private args: GhostArgs | null = null;
  private model = '';
  private template: ModelTemplate | null = null;
  /** A draw per mesh of the template, and each mesh's own material (shown where the thing fits). */
  private draws: Instances[] = [];
  private own: pc.Material[] = [];
  /** The translucent tint where it can't go, and the arrow's material and draw. */
  /** The red copy's materials: for meshes without vertex colours, and with them (`shaders/ghost.ts`). */
  private readonly redMaterials: [pc.ShaderMaterial, pc.ShaderMaterial];
  /** The red copy's light (game space, the look's units unconverted: `light`). */
  private readonly sunDir = new Float32Array(3);
  private readonly sun = new Float32Array(3);
  private readonly sky = new Float32Array(3);
  private readonly ground = new Float32Array(3);

  private readonly arrowMaterial: pc.ShaderMaterial;
  /** The red copy's alpha (x) and the arrow's colour (linear) and alpha: uniforms written in place. */
  private readonly redAlpha = Float32Array.of(0.36, 0, 0, 0);
  private readonly arrowColor = new Float32Array(4);
  private readonly arrow: Instances;
  private arrowOn = false;
  private valid = true;
  /** Whether it's in hand, and its goal: the footprint's middle, the lift, the turn. */
  shown = false;
  private readonly goal = { x: 0, z: 0 };
  private lift = 0;
  private yawGoal = 0;
  /**
   * What moves each frame, in a typed array (a number field written each frame would box its
   * value): its seconds in hand and since it sprang in, where it is now (it glides: lot rows,
   * above its storey's floor), its turn, its alpha (`AGE` … `ALPHA`).
   */
  private readonly now = new Float64Array(7);
  /** The arrow's place and turn in the ghost's turned frame. */
  private readonly arrowAt = { x: 0, y: 0, z: 0, yaw: 0 };
  private destroyed = false;
  /** Reduce motion: held still, no spring, no pulse. */
  reducedMotion = false;
  /** The ghost's matrix (its zeros and its 1 stay; `frame` writes the rest). */
  private readonly matrix = Float32Array.of(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);
  private readonly arrowLocal = new Float32Array(16);
  private readonly arrowMatrix = new Float32Array(16);
  private readonly q: Quat = [0, 0, 0, 1];

  constructor(
    private readonly app: pc.AppBase,
    /** The storey layer (lot rows, lifted to the storey in view). */
    private readonly parent: pc.Entity,
    camera: pc.CameraComponent,
    private readonly deps: RendererDeps,
  ) {
    this.device = app.graphicsDevice;
    this.layer = overLayer(app, camera).id;
    // The red Blinn-Phong copy (`shaders/ghost.ts`), both sides, blended, no depth written.
    const h = Math.hypot(0.2, 1, -0.3);
    const hemi = new Float32Array([0.2 / h, 1 / h, -0.3 / h]);
    this.redMaterials = [false, true].map((colors) => {
      const red = new pc.ShaderMaterial({
        uniqueName: 'GameGhost',
        vertexGLSL: GHOST_VS.glsl,
        fragmentGLSL: GHOST_PS.glsl,
        vertexWGSL: GHOST_VS.wgsl,
        fragmentWGSL: GHOST_PS.wgsl,
        attributes: { vertex_position: pc.SEMANTIC_POSITION, vertex_normal: pc.SEMANTIC_NORMAL, vertex_color: pc.SEMANTIC_COLOR, instance_line1: pc.SEMANTIC_ATTR11, instance_line2: pc.SEMANTIC_ATTR12, instance_line3: pc.SEMANTIC_ATTR14, instance_line4: pc.SEMANTIC_ATTR15 },
      });
      red.name = colors ? 'ghostColored' : 'ghost';
      if (colors) red.setDefine('GAME_GHOST_COLORS', '');
      red.blendType = pc.BLEND_NORMAL;
      red.depthWrite = false;
      red.cull = pc.CULLFACE_NONE;
      // (The sky's hemispheric light comes from (0.2, 1, -0.3), normalised, as the materials' sky fill, `SKY_FILL_DIRECTION`.)
      red.setParameter('gameGhostHemiDir', hemi);
      red.setParameter('gameGhostAlpha', this.redAlpha);
      red.setParameter('gameGhostSunDir', this.sunDir);
      red.setParameter('gameGhostSun', this.sun);
      red.setParameter('gameGhostSky', this.sky);
      red.setParameter('gameGhostGround', this.ground);
      red.update();
      return red;
    }) as [pc.ShaderMaterial, pc.ShaderMaterial];
    // Unlit, in its own colour (`tint`), both sides, no depth written.
    const arrowMat = (this.arrowMaterial = new pc.ShaderMaterial({
      uniqueName: 'GameGhostArrow',
      vertexGLSL: GHOST_VS.glsl,
      fragmentGLSL: ARROW_PS.glsl,
      vertexWGSL: GHOST_VS.wgsl,
      fragmentWGSL: ARROW_PS.wgsl,
      attributes: { vertex_position: pc.SEMANTIC_POSITION, vertex_normal: pc.SEMANTIC_NORMAL, instance_line1: pc.SEMANTIC_ATTR11, instance_line2: pc.SEMANTIC_ATTR12, instance_line3: pc.SEMANTIC_ATTR14, instance_line4: pc.SEMANTIC_ATTR15 },
    }));
    arrowMat.name = 'ghostArrow';
    arrowMat.blendType = pc.BLEND_NORMAL;
    arrowMat.depthWrite = false;
    arrowMat.cull = pc.CULLFACE_NONE;
    arrowMat.setParameter('gameArrowColor', this.arrowColor);
    arrowMat.update();
    // A stem and a head lying flat, pointing +z (out of the object's front), within one tile.
    const mesh = new pc.Mesh(this.device);
    mesh.setPositions([-0.09, 0, -0.36, 0.09, 0, -0.36, 0.09, 0, 0.04, -0.09, 0, 0.04, -0.26, 0, 0.04, 0.26, 0, 0.04, 0, 0, 0.38]);
    mesh.setNormals(Array.from({ length: 7 }, () => [0, 1, 0]).flat());
    mesh.setIndices([0, 2, 1, 0, 3, 2, 4, 6, 5]);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    this.arrow = this.drawOf(mesh, arrowMat, 'ghostArrow');
    this.tint();
    mesh.decRefCount();
  }

  /** The red copy's materials, without and with vertex colours (the renderer's warm-up draws them once). */
  get redCopy(): readonly [pc.Material, pc.Material] {
    return this.redMaterials;
  }

  /** The thing in hand, or none (`Renderer.setPlacementGhost`). Cheap: every pointer move. */
  set(ghost: PlacementGhost | null): void {
    if (!ghost) {
      if (this.shown) {
        this.shown = false;
        this.show();
      }
      return;
    }
    // A new thing in hand pops in; the same one resized just changes shape.
    const fresh = !this.shown || ghost.model !== this.model;
    if (ghostKeyOf(ghost.model, ghost.def, ghost.resize, ghost.colors) !== this.key) this.load(ghost.model, ghost.def, ghost.resize, ghost.colors);
    // Rotate about the footprint centre (moved by its offset), like placed objects. The turn takes the short way round.
    const centre = objectCentre(ghost);
    this.goal.x = centre.x;
    this.goal.z = centre.z;
    this.lift = ghost.lift ?? 0;
    const yaw = (ghost.rot * Math.PI) / 2 + ((ghost.turn ?? 0) * Math.PI) / 180;
    const turn = Math.atan2(Math.sin(yaw - this.yawGoal), Math.cos(yaw - this.yawGoal));
    this.yawGoal += turn;
    if (!this.shown) {
      this.now[X] = this.goal.x;
      this.now[Y] = 0;
      this.now[Z] = this.goal.z;
      this.now[YAW] = this.yawGoal = yaw;
    }
    if (fresh) this.now[SPAWN] = 0;
    const was = this.shown;
    this.shown = true;
    // The arrow rides with the ghost but ignores its free turn: the use tile goes by quarter turns
    // (a seat used from beside or behind: it points away from the seat there).
    this.arrowOn = !!ghost.front;
    if (ghost.front) {
      const dx = ghost.front.x + 0.5 - this.goal.x;
      const dz = ghost.front.z + 0.5 - this.goal.z;
      const c = Math.cos(yaw);
      const s = Math.sin(yaw);
      // World offset into the ghost's turned frame; just above the floor below the floating ghost.
      this.arrowAt.x = dx * c - dz * s;
      this.arrowAt.y = -0.04;
      this.arrowAt.z = dx * s + dz * c;
      this.arrowAt.yaw = ((ghost.front.out - ghost.rot) * Math.PI) / 2 - ((ghost.turn ?? 0) * Math.PI) / 180;
      const a = this.arrowAt;
      compose(1, 1, 1, quatYawPitchRoll(a.yaw, 0, 0, this.q), a.x, a.y, a.z, this.arrowLocal);
    }
    if (ghost.valid !== this.valid) {
      this.valid = ghost.valid;
      this.tint();
    }
    if (!was) this.show();
    else this.arrow.enabled = this.arrowOn;
  }

  /**
   * Each game frame while it's in hand (`dtMs`): it glides to its goal and turns, floats and bobs,
   * springs in, and pulses where it can't go.
   */
  frame(dtMs: number): void {
    if (!this.shown) return;
    const dt = Math.min(dtMs, 50) / 1000;
    const now = this.now;
    now[AGE] += dt;
    now[SPAWN] += dt;
    const glide = 1 - Math.exp(-dt * 22);
    now[X] += (this.goal.x - now[X]) * glide;
    now[Z] += (this.goal.z - now[Z]) * glide;
    now[YAW] += (this.yawGoal - now[YAW]) * (1 - Math.exp(-dt * 16));
    // Held a hand's breadth above the floor (or the top it goes on), bobbing gently; picked up with a springy pop.
    const still = this.reducedMotion;
    now[Y] = this.lift + 0.07 + (still ? 0 : 0.035 * Math.sin(now[AGE] * 3.4));
    const t = now[SPAWN];
    const s = still || t > 0.6 ? 1 : 1 - Math.exp(-8 * t) * Math.cos(14 * t);
    // Where it can't go, it pulses.
    now[ALPHA] = this.valid ? 0.5 : still ? 0.36 : 0.36 + 0.14 * Math.sin(now[AGE] * 9);
    this.redAlpha[0] = now[ALPHA];
    this.arrowColor[3] = now[ALPHA] + 0.25;
    // (Scale, turn about y and place, written out: a helper given doubles boxes each one until it's inlined.)
    const m = this.matrix;
    const c = Math.cos(now[YAW]) * s;
    const sn = Math.sin(now[YAW]) * s;
    m[0] = c;
    m[2] = -sn;
    m[5] = s;
    m[8] = sn;
    m[10] = c;
    m[12] = now[X];
    m[13] = now[Y];
    m[14] = now[Z];
    for (const draw of this.draws) {
      draw.setMatrix(0, m);
      draw.update(false);
    }
    if (this.arrowOn) {
      // (Its local matrix is made when it's placed: `set`.)
      this.arrow.setMatrix(0, multiply(this.arrowLocal, m, this.arrowMatrix));
      this.arrow.update(false);
    }
  }

  /**
   * The light the red copy is shaded in (each lighting update): the sun (towards it) and the sky's
   * colours at their strengths, in game space and the look's units as they are (`shaders/ghost.ts`).
   */
  light(l: Lighting): void {
    const d = l.sunDirection;
    const len = Math.hypot(d.x, d.y, d.z) || 1;
    setScaled(this.sunDir, d.x, d.y, d.z, -1 / len);
    setScaled(this.sun, l.sunColor.r, l.sunColor.g, l.sunColor.b, l.sunIntensity);
    setScaled(this.sky, l.skyColor.r, l.skyColor.g, l.skyColor.b, l.skyIntensity);
    setScaled(this.ground, l.groundColor.r, l.groundColor.g, l.groundColor.b, l.skyIntensity);
  }

  /** A model file that failed has come: its models are built with it again, and the thing in hand, if it's one, now (its box stays until then). */
  fileArrived(url: string): void {
    // (The thing in hand is a box while its model is unfiled.)
    const inHand = !!this.unfiled.get(this.model)?.includes(url);
    for (const [model, files] of this.unfiled) if (files.includes(url)) this.unfiled.delete(model);
    if (!inHand || !this.args) return;
    if (this.shown) this.load(...this.args);
    // Put away: built afresh when next taken in hand.
    else this.key = '';
  }

  destroy(): void {
    this.destroyed = true;
    this.dropModel();
    // (The arrow's draw holds its mesh: it goes with it.)
    this.arrow.destroy();
    for (const red of this.redMaterials) red.destroy();
    this.arrowMaterial.destroy();
  }

  /** One copy of `mesh` drawn over the world, hidden. */
  private drawOf(mesh: pc.Mesh, material: pc.Material, name: string): Instances {
    const draw = new Instances(this.device, this.parent, mesh, material, { name, castShadows: false, receiveShadows: false, layers: [this.layer], cull: false });
    draw.setCount(1);
    draw.enabled = false;
    return draw;
  }

  /** Builds (once per model key) the ghost's copy of the thing: the thing itself, held up. */
  private load(model: string, kind: string | undefined, resize?: ObjectResize, colors?: PartColors): void {
    const key = ghostKeyOf(model, kind, resize, colors);
    this.key = key;
    // Another thing: the old one goes now. The same thing resized keeps its old shape until the new one is built.
    if (model !== this.model) this.dropModel();
    this.model = model;
    this.args = [model, kind, resize, colors];
    // Without its file, the box the placed thing would get. A file that failed for the ghost isn't
    // fetched again for each step of a resize or a colour: the box is built straight away, until the
    // file comes (`fileArrived`).
    const content = this.deps.content;
    const def = kind === undefined ? undefined : content.object(kind);
    const unfiled = this.unfiled.get(model);
    const entry = unfiled ? undefined : painted(this.deps.assets.get(model, 'model'), colors);
    void buildModel(this.app, `ghost:${key}`, entry, resize?.from ?? def?.footprint ?? [1, 1], { resize, standIn: standInOf(content, def) }).then((template) => {
      if (template.failed) this.unfiled.set(model, template.failed);
      // The ghost doesn't glow.
      for (const g of template.glows ?? []) g.mesh.mesh.destroy();
      if (this.destroyed || this.key !== key) {
        disposeTemplate({ meshes: template.meshes });
        return;
      }
      this.dropModel();
      this.template = template;
      this.draws = template.meshes.map((m) => this.drawOf(m.mesh, m.material, `ghost:${m.name}`));
      this.own = template.meshes.map((m) => m.material);
      this.tint();
      this.show();
    });
  }

  /** Lets go of the ghost's copy of the model (its draws, then its meshes). */
  private dropModel(): void {
    for (const draw of this.draws) draw.destroy();
    if (this.template) disposeTemplate({ meshes: this.template.meshes });
    this.draws = [];
    this.own = [];
    this.template = null;
  }

  /** Where it fits, the thing itself (and a green arrow); where it can't go, a red see-through copy. */
  private tint(): void {
    // (The arrow's colour in gamma space, taken to linear for the image processing that follows.)
    const [r, g, b] = this.valid ? [0.55, 1.0, 0.6] : [1.0, 0.45, 0.42];
    this.arrowColor[0] = toLinear(r);
    this.arrowColor[1] = toLinear(g);
    this.arrowColor[2] = toLinear(b);
    this.draws.forEach((draw, i) => {
      draw.material = this.valid ? this.own[i] : this.redMaterials[hasColors(draw.mesh) ? 1 : 0];
    });
  }

  /** Its draws on or off (in hand or not), placed for this frame. */
  private show(): void {
    for (const draw of this.draws) draw.enabled = this.shown;
    this.arrow.enabled = this.shown && this.arrowOn;
    if (this.shown) this.frame(0);
  }
}
