/**
 * Mirror reflections on PlayCanvas. Every mirror's glass (a model's `mirrors`, `LotContents.mirrors`)
 * gets a flat stand-in just in front of it; the few nearest the camera that are on screen and face it
 * show the room in a render target of their own, the scene drawn once more by a reflection camera, at
 * a share of the screen's size. The rest keep their glossy
 * glass. A reflection is only drawn while its mirror is on screen.
 *
 * - The reflection camera: the game camera's view reflected in the glass's plane (`calculateTransform`:
 *   PlayCanvas space, a mirror's matrix, so `flipFaces`), and the game camera's projection with its
 *   near plane turned into the glass's plane (an oblique near plane, Lengyel's, in `calculateProjection`;
 *   rather than a clip plane: no shader variants), narrowed to the glass's rectangle on screen and
 *   drawn only there (the camera's `rect`). So it draws what can be seen through the glass and
 *   nothing behind it: its frustum is the pyramid from the reflected eye through the glass. It draws
 *   the world layer in linear light into a half-float target (no tone mapping, no gamma: the scene's
 *   own picture is linear), multisampled as the preset says, its size a share of the drawing's
 *   rounded to powers of two (`size`).
 * - Left out of every reflection: the glasses themselves and the build helpers, by name (`HELPER`),
 *   hidden from the reflection cameras' culling only.
 * - Residents: drawn in a reflection only if they stand where its glass shows them: the resident
 *   shader skips the rows not set in `game_simShown` (`residentRowFilter`), which is set for each
 *   reflection camera's layers and put back after.
 * - The sun's shadow map: PlayCanvas keeps a directional light's matrices per camera. The reflection
 *   cameras use the game camera's (`getRenderData` on the sun's light), so a reflection drawn after
 *   frames without one still has the map's matrices of now, and the map is drawn at most once a frame
 *   for the game camera and its reflections together (`getLightRenderPass`).
 * - The glass: unlit, the reflection looked up through the game camera's view × projection when it was
 *   drawn (`game_mirrorLookup`, world position to the target's texture coordinates): the reflection
 *   in gamma space times 0.92, plus a hint of the glass's own tint, taken back to linear (the image
 *   processing comes after, on the whole picture).
 *
 * With a `rate` (Low and Medium) a reflection is drawn on demand and held in between ("held": its
 * camera is off and its target keeps the last picture): again when the camera moves, the mirror moves
 * (a cabinet's door), a resident in its view moves, or the renderer says something else changed
 * (`changed`: the sun, the lamps, things placed or moving), at most `rate` times a second. A held
 * reflection is looked up through the camera it was drawn for, so it stays on the glass while the
 * camera moves on (only its parallax lags, until the next draw). The picking that places the glasses
 * and ranks them runs only on the frames that may draw one. Without a rate (High) every reflection is
 * drawn every frame.
 *
 * Mirrors follow their object's instance matrix each frame, so things popping in, hidden storeys,
 * things on walls cut away and objects being moved need nothing extra. A reflection's target (and
 * camera) exists only while a mirror uses it (and 10 s after). Per frame: no allocations.
 */

import * as pc from './engine';
import { directionalShadows } from './internals';
import { residentRowFilter, RESIDENT_ROWS_ALL } from './characterMaterial';
import { Instances } from './instancing';
import type { MirrorInstance } from './lotContents';
import { multiply, transformNormal, transformPoint } from '../shared/math';
import type { Language } from './shaders/chunks';

export type { MirrorInstance } from './lotContents';

/** How the mirrors reflect (a preset's `mirrors`, `mirrorScale`, `mirrorRate`; `msaaSamples`). */
export interface MirrorQuality {
  /** How many mirrors reflect at once (0: none). */
  count: number;
  /** Resolution of a reflection, as a share of the screen's along each side. */
  scale: number;
  /** Multisampling of a reflection (1: none). */
  samples: number;
  /** At most this many draws of a reflection a second, each when something in it changed; 0: every frame. */
  rate: number;
}

/** Residents as the characters last posed them (`characters.ts`): head tops (x, y, z per row, game space), which are drawn, whether any moved this frame. */
export interface MirrorResidents {
  readonly heads: Float32Array;
  readonly visible: Uint8Array;
  readonly posed: boolean;
}

/** Mirrors further from the camera than this (m) aren't worth a reflection. */
const RANGE = 40;
/** Milliseconds a reflection's render target is kept once no mirror uses it. */
const KEEP_MS = 10_000;
/** Metres a resident's box may lie outside a mirror's view and still be drawn (the eye moves on a little before the draw). */
const MARGIN = 0.3;
/** A resident's box round their head top: half its width, and how far it reaches down (m). */
const RESIDENT_HALF = 0.4;
const RESIDENT_DOWN = 1.9;
/** Milliseconds a draw may come before it is due (frames don't land on the time exactly). */
const EARLY = 4;
/** The camera counts as moved past this change in any of its view × projection's numbers. */
const STILL = 1e-5;
/** Target texels round the glass's rectangle drawn too (the lookup filters across its edge). */
const PAD = 2;
/** The glass's own tint, added to the reflection it passes (0.92 of it, in the glass's shader), gamma space. */
const TINT = new Float32Array([0.006, 0.008, 0.009]);
/** Draws a reflection never shows: the glasses and the build helpers, by their names. */
const HELPER = /^(mirror:|cursor$|resize:|buildGrid$|preview-|room-overlay$|marker)/;

/** The glass's emissive: the reflection through the drawn view (see the file's header). */
const GLASS_EMISSIVE: Record<Language, string> = {
  glsl: `
uniform sampler2D game_mirror;
uniform mat4 game_mirrorLookup;
uniform vec3 game_mirrorTint;
void getEmission() {
	vec4 gameMirrorClip = game_mirrorLookup * vec4(vPositionW, 1.0);
	vec3 gameMirror = texture2D(game_mirror, gameMirrorClip.xy / gameMirrorClip.w).rgb;
	dEmission = pow(pow(max(gameMirror, vec3(0.0)), vec3(1.0 / 2.2)) * 0.92 + game_mirrorTint, vec3(2.2));
}
`,
  wgsl: `
var game_mirror: texture_2d<f32>;
var game_mirrorSampler: sampler;
uniform game_mirrorLookup: mat4x4f;
uniform game_mirrorTint: vec3f;
fn getEmission() {
	let gameMirrorClip = uniform.game_mirrorLookup * vec4f(vPositionW, 1.0);
	let gameMirror = textureSample(game_mirror, game_mirrorSampler, gameMirrorClip.xy / gameMirrorClip.w).rgb;
	dEmission = pow(pow(max(gameMirror, vec3f(0.0)), vec3f(1.0 / 2.2)) * 0.92 + uniform.game_mirrorTint, vec3f(2.2));
}
`,
};

interface Reflection {
  entity: pc.Entity;
  camera: pc.CameraComponent;
  target: pc.RenderTarget;
  material: pc.StandardMaterial;
  /** World (PlayCanvas space) to the target's texture coordinates, as drawn (`game_mirrorLookup`, written in place). */
  lookup: Float32Array;
  /** The reflection camera's inverse view and projection for its draw (`calculateTransform`, `calculateProjection`). */
  viewInv: pc.Mat4;
  proj: pc.Mat4;
  /** Its viewport in the target (normalised). */
  rect: pc.Vec4;
  /** When a mirror last used it (`performance.now()`). */
  used: number;
  /** The mirror it shows, and its face and the game camera's view × projection (game space) when last drawn. */
  owner: Item | null;
  drawnFace: Float32Array;
  drawnViewProj: Float32Array;
  /** When the next draw is due (`performance.now()`), held to `rate`. */
  due: number;
  /** Who it is being drawn for now, and something in it changed since it was drawn. */
  next: Item | null;
  dirty: boolean;
  /** Drawn this frame (its camera on). */
  drawing: boolean;
  /** What can be seen through the glass: planes (x, y, z, d each, inside positive, game space) a box must reach into. */
  planes: Float32Array;
  planeCount: number;
  /** The resident rows it shows (`game_simShown`: 16 bits a component), and whether anyone. */
  shows: Float32Array;
  showsAnyone: boolean;
}

interface Item {
  instance: MirrorInstance;
  /** A draw per glass face (the unit square or disc, its matrix the face's). */
  draws: Instances[];
  /** Pool slot showing its reflection, or -1. */
  slot: number;
  /** Ranked this frame (among the nearest on screen). */
  chosen: boolean;
  /** Its object's instance matrix this frame, and its first face's world matrix (game space). */
  matrix: Float32Array;
  face: Float32Array;
  centre: Float32Array;
  normal: Float32Array;
  /** The glass's corners round all its faces, in the first face's plane (game space, x y z each). */
  corners: Float32Array;
  distance: number;
  /** Per face: the world matrix its draw has. */
  worlds: Float32Array[];
}

/** The unit square's corners, in order round it. */
const SQUARE = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]] as const;

export class Mirrors {
  private readonly device: pc.GraphicsDevice;
  private readonly pool: (Reflection | null)[];
  private readonly square: pc.Mesh;
  private readonly disc: pc.Mesh;
  /** The glass of a mirror not reflecting (its draw is off), and its black picture. */
  private readonly idle: pc.StandardMaterial;
  private readonly black: pc.Texture;
  private items: Item[] = [];
  private instances: MirrorInstance[] = [];
  /** Something the reflections show changed (see `changed`). */
  private stale = false;
  /** The glasses are off (the menus' town, or nothing to reflect). */
  private hidden = true;
  /** The reflection cameras (PlayCanvas's scene cameras), for the sun's matrices. */
  private readonly cameras = new Set<pc.Camera>();
  /** The resident rows the resident shader draws (`residentRowFilter`). */
  private readonly rowFilter: Float32Array;
  /** Per mesh instance: left out of reflections (`HELPER`), and those hidden for the cull now. */
  private readonly helpers = new WeakMap<pc.MeshInstance, boolean>();
  private readonly hiddenNow: pc.MeshInstance[] = [];
  private worldLayer: pc.Layer | null = null;
  private readonly unpatch: (() => void)[] = [];
  /** The frame (`app.frame`) the sun's map was last given a pass in, for the game camera and the reflections. */
  private sunPassFrame = -1;
  // Scratch (no allocations per frame).
  private readonly viewProj = new pc.Mat4();
  private readonly viewProjGame = new Float32Array(16);
  private readonly view = new pc.Mat4();
  private readonly frustum = new Float32Array(24);
  private readonly mirror = new pc.Mat4();
  private readonly ranked: Item[] = [];
  private readonly free: boolean[];
  private readonly eye = { x: 0, y: 0, z: 0 };
  private readonly eyeImage = new Float32Array(3);
  private readonly world = new Float32Array(16);
  private readonly p = new Float32Array(3);
  private readonly u = new Float32Array(3);
  private readonly v = new Float32Array(3);
  private readonly lo = new Float32Array(3);
  private readonly hi = new Float32Array(3);
  private readonly ndc = new Float32Array(4);
  private readonly onResize = () => this.resized();
  private readonly onPrecull = (camera: pc.CameraComponent | null) => this.precull(camera);
  private readonly onPostcull = (camera: pc.CameraComponent | null) => this.postcull(camera);
  private readonly onLayer = (camera: pc.CameraComponent, layer: pc.Layer) => this.layerStarts(camera, layer);
  private readonly onLayerDone = (camera: pc.CameraComponent, layer: pc.Layer) => this.layerEnds(camera, layer);
  /** Rows of the residents this frame. */
  private residentCount = 0;
  /** Reflections drawn so far (the benchmark's count). */
  draws = 0;
  /** The warm-up's frame (`warm`): the reflections draw everything, unculled. */
  private warming = false;

  /**
   * Mirrors in `app`'s world (`root`, game space), seen by `game` (the game camera: its entity's pose
   * and its projection), lit by `sun` (its shadow map shared with the reflections).
   */
  constructor(
    private readonly app: pc.AppBase,
    private readonly root: pc.Entity,
    private readonly game: pc.CameraComponent,
    sun: pc.LightComponent,
    private readonly quality: MirrorQuality,
    /** Whether a resident draw draws any of the rows set in `bits` (null: not a resident part; `Characters.drawsRows`). */
    private readonly drawsRows: (mi: pc.MeshInstance, bits: Float32Array) => boolean | null = () => null,
  ) {
    const device = (this.device = app.graphicsDevice);
    this.pool = Array.from({ length: Math.max(0, quality.count) }, () => null);
    this.free = this.pool.map(() => true);
    this.square = unitMesh(device, false);
    this.disc = unitMesh(device, true);
    this.black = new pc.Texture(device, { name: 'mirrorNone', width: 1, height: 1, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, levels: [new Uint8Array(4)] });
    this.idle = glassMaterial(this.black, new Float32Array(16));
    this.rowFilter = residentRowFilter(app);
    this.worldLayer = app.scene.layers.getLayerById(pc.LAYERID_WORLD);
    device.on('resizecanvas', this.onResize);
    const scene = app.scene;
    scene.on(pc.EVENT_PRECULL, this.onPrecull);
    scene.on(pc.EVENT_POSTCULL, this.onPostcull);
    scene.on(pc.EVENT_PRERENDER_LAYER, this.onLayer);
    scene.on(pc.EVENT_POSTRENDER_LAYER, this.onLayerDone);
    if (this.pool.length) this.shareSunShadows(sun);
  }

  /**
   * The reflection cameras take the game camera's matrices of the sun's map (PlayCanvas keeps them per
   * camera), and the map is drawn at most once a frame for all of them: the reflections are drawn
   * first, so the first pass of the frame draws it for every camera after.
   */
  private shareSunShadows(sun: pc.LightComponent): void {
    // SAFETY: a light's render data per camera (`Light.getRenderData`) and the forward renderer's
    // directional shadow renderer (`_shadowRendererDirectional.getLightRenderPass`) are PlayCanvas
    // 2.23's (pinned); `check:chunks` checks both are still there. Own properties on the instances
    // shadow the methods, and are deleted again on `destroy`.
    const light = sun.light as unknown as { getRenderData(camera: pc.Camera | null, face: number): unknown };
    const own = light.getRenderData;
    light.getRenderData = (camera, face) => own.call(light, camera && this.cameras.has(camera) ? this.game.camera : camera, face);
    this.unpatch.push(() => delete (light as { getRenderData?: unknown }).getRenderData);
    const sr = directionalShadows(this.app);
    if (!sr) return;
    const pass = sr.getLightRenderPass;
    sr.getLightRenderPass = (l, camera) => {
      if (l !== light || (camera !== this.game.camera && !this.cameras.has(camera))) return pass.call(sr, l, camera);
      if (this.sunPassFrame === this.app.frame) return null;
      const made = pass.call(sr, l, camera);
      if (made) this.sunPassFrame = this.app.frame;
      return made;
    };
    this.unpatch.push(() => delete (sr as { getLightRenderPass?: unknown }).getLightRenderPass);
  }

  /** The reflection in slot `i` (made the first time: its target, camera and glass). */
  private reflection(i: number): Reflection {
    const known = this.pool[i];
    if (known) return known;
    const device = this.device;
    const [width, height] = this.size();
    const texture = new pc.Texture(device, {
      name: `mirror${i}`,
      width,
      height,
      format: device.getRenderableHdrFormat([pc.PIXELFORMAT_RGBA16F], true) ?? pc.PIXELFORMAT_RGBA8,
      mipmaps: false,
      minFilter: pc.FILTER_LINEAR,
      magFilter: pc.FILTER_LINEAR,
      addressU: pc.ADDRESS_CLAMP_TO_EDGE,
      addressV: pc.ADDRESS_CLAMP_TO_EDGE,
    });
    const target = new pc.RenderTarget({ name: `mirror${i}`, colorBuffer: texture, depth: true, samples: Math.max(1, this.quality.samples) });
    const entity = new pc.Entity(`mirrorCamera${i}`);
    // (Before the game camera and the save's picture: the glass shows this frame's reflection.)
    entity.addComponent('camera', {
      priority: this.game.priority - 2,
      layers: [pc.LAYERID_WORLD],
      clearColor: new pc.Color(0, 0, 0, 1),
      toneMapping: pc.TONEMAP_NONE,
      gammaCorrection: pc.GAMMA_NONE,
      renderTarget: target,
      flipFaces: true,
    });
    const camera = entity.camera!;
    const lookup = new Float32Array(16);
    const r: Reflection = {
      entity,
      camera,
      target,
      material: glassMaterial(texture, lookup),
      lookup,
      viewInv: new pc.Mat4(),
      proj: new pc.Mat4(),
      rect: new pc.Vec4(0, 0, 1, 1),
      used: 0,
      owner: null,
      drawnFace: new Float32Array(16),
      drawnViewProj: new Float32Array(16),
      due: -Infinity,
      next: null,
      dirty: true,
      drawing: false,
      planes: new Float32Array(20),
      planeCount: 0,
      shows: new Float32Array(4),
      showsAnyone: false,
    };
    camera.calculateTransform = (out) => out.copy(r.viewInv);
    camera.calculateProjection = (out) => out.copy(r.proj);
    entity.enabled = false;
    this.app.root.addChild(entity);
    this.cameras.add(camera.camera);
    return (this.pool[i] = r);
  }

  /**
   * A reflection's size for its ratio of the drawing (`QualitySettings.mirrorScale`): each side
   * scaled, eased up for small sizes and rounded to the nearest power of two, at most the drawing's
   * own (its power of two below).
   */
  private size(): [number, number] {
    const best = (dimension: number) => {
      const x = dimension * this.quality.scale;
      const curved = nearestPOT(x + (128 * 128) / (128 + x));
      return Math.max(1, Math.min(floorPOT(dimension), curved));
    };
    return [best(this.device.width), best(this.device.height)];
  }

  /** A resized drawing: the reflections are resized and start again. */
  private resized(): void {
    const [width, height] = this.size();
    for (const r of this.pool) if (r && (r.target.width !== width || r.target.height !== height)) r.target.resize(width, height);
    this.changed();
  }

  /** Lets go of the reflections no mirror has used for `KEEP_MS` (all of them with `now` Infinity). */
  private release(now: number): void {
    for (let i = 0; i < this.pool.length; i++) {
      const r = this.pool[i];
      if (!r || now - r.used < KEEP_MS || this.items.some((item) => item.slot === i)) continue;
      for (const item of this.items) for (const d of item.draws) if (d.material === r.material) d.material = this.idle;
      this.cameras.delete(r.camera.camera);
      r.entity.destroy();
      r.material.destroy();
      r.target.destroyTextureBuffers();
      r.target.destroy();
      this.pool[i] = null;
    }
  }

  /** The mirrors of the world being shown (replaces the last ones). */
  set(instances: readonly MirrorInstance[]): void {
    for (const item of this.items) for (const d of item.draws) d.destroy();
    this.items = [];
    this.ranked.length = 0;
    this.instances = instances.filter((i) => i.surfaces.length);
    for (const r of this.pool) if (r) r.owner = r.next = null;
    // A world without mirrors needs none of the reflections.
    if (!this.instances.length) this.release(Infinity);
    if (!this.pool.length) return;
    for (const instance of this.instances) {
      const draws = instance.surfaces.map((s, i) => {
        const d = new Instances(this.device, this.root, s.round ? this.disc : this.square, this.idle, { name: `mirror:${instance.index}:${i}`, castShadows: false, receiveShadows: false, cull: false });
        d.setCount(1);
        d.enabled = false;
        return d;
      });
      this.items.push({
        instance,
        draws,
        slot: -1,
        chosen: false,
        matrix: new Float32Array(16),
        face: new Float32Array(16),
        centre: new Float32Array(3),
        normal: new Float32Array(3),
        corners: new Float32Array(12),
        distance: 0,
        worlds: draws.map(() => new Float32Array(16)),
      });
    }
    this.hidden = false;
    this.changed();
  }

  /** Something the reflections show changed (the light, lamps, things placed or moving): held ones are drawn again. */
  changed(): void {
    this.stale = true;
  }

  /**
   * Per frame, before it is drawn (the game camera posed): places the mirrors, gives the reflections
   * to the nearest ones on screen facing the camera, and says which to draw now. `residents`: as just
   * posed (their first `count` rows). `off`: none drawn (the menus' town).
   */
  frame(residents: MirrorResidents | null, count: number, off = false): void {
    for (const r of this.pool) if (r) r.drawing = false;
    this.residentCount = Math.min(count, 64);
    this.pick(residents, off);
    for (const r of this.pool) if (r && r.entity.enabled !== r.drawing) r.entity.enabled = r.drawing;
  }

  private pick(residents: MirrorResidents | null, off: boolean): void {
    if (off || !this.items.length) {
      this.hide();
      this.stale = false;
      return;
    }
    this.hidden = false;
    const now = performance.now();
    this.viewProjNow();
    const viewProj = this.viewProjGame;
    const period = this.quality.rate > 0 ? 1000 / this.quality.rate : 0;
    // A reflection draws again at most `rate` times a second: until one is due, nothing it shows
    // can have changed in a way that matters, so the glasses keep their place and their held
    // reflection, and the picking that would re-place them waits for the frame that draws.
    if (period && !this.stale && !this.due(now, viewProj, residents)) return;
    const cam = this.game.entity;
    const p = cam.getPosition();
    const eye = this.eye;
    eye.x = p.x;
    eye.y = p.y;
    eye.z = -p.z;
    planesOf(viewProj, this.frustum);
    const ranked = this.ranked;
    ranked.length = 0;
    for (const item of this.items) {
      item.chosen = false;
      const { matrices, index, surfaces } = item.instance;
      for (let k = 0; k < 16; k++) item.matrix[k] = matrices[index * 16 + k];
      multiply(surfaces[0].matrix, item.matrix, item.face);
      transformPoint(item.face, 0, 0, 0, item.centre);
      transformNormal(item.face, 0, 0, 1, item.normal);
      const c = item.centre;
      const n = item.normal;
      const length = Math.hypot(n[0], n[1], n[2]);
      item.distance = Math.hypot(eye.x - c[0], eye.y - c[1], eye.z - c[2]);
      // Hidden (scaled away), turned away from the camera, too far to matter, or off screen.
      if (length < 1e-6) continue;
      n[0] /= length;
      n[1] /= length;
      n[2] /= length;
      const facing = (eye.x - c[0]) * n[0] + (eye.y - c[1]) * n[1] + (eye.z - c[2]) * n[2];
      if (facing <= 0 || item.distance >= RANGE) continue;
      this.outline(item);
      if (this.onScreen(item)) ranked.push(item);
    }
    ranked.sort(byDistance);
    for (let i = 0; i < ranked.length && i < this.pool.length; i++) ranked[i].chosen = true;
    // Keep each chosen mirror's slot where it can (no flicker as the order changes).
    this.free.fill(true);
    for (const item of this.items) {
      if (item.chosen && item.slot >= 0) this.free[item.slot] = false;
      else item.slot = -1;
    }
    for (const item of this.items) {
      if (item.chosen && item.slot < 0) {
        item.slot = this.free.indexOf(true);
        this.free[item.slot] = false;
      }
      const on = item.slot >= 0;
      let r: Reflection | null = null;
      if (on) {
        r = this.reflection(item.slot);
        r.used = now;
        r.next = item;
        this.view5(item, r);
        this.seesResidents(r, residents);
      }
      const { surfaces } = item.instance;
      for (let i = 0; i < item.draws.length; i++) {
        const d = item.draws[i];
        if (d.enabled !== on) d.enabled = on;
        if (!r) continue;
        if (d.material !== r.material) d.material = r.material;
        const w = multiply(surfaces[i].matrix, item.matrix, this.world);
        if (!same(w, item.worlds[i], 0)) {
          item.worlds[i].set(w);
          d.setMatrix(0, w);
          d.update(false);
        }
      }
      if (!r) continue;
      // Every frame (High); held: drawn again at once for another mirror; when something in it changed, once it's due.
      if (!period || r.owner !== item) {
        this.draw(r, item);
        continue;
      }
      if (this.stale || !same(item.face, r.drawnFace, STILL) || this.seenMoving(r, viewProj, residents)) r.dirty = true;
      if (r.dirty && now >= r.due - EARLY) {
        // Steady at `rate` while it keeps changing; after a still spell, not two draws in a row.
        r.due = Math.max(r.due + period, now + period * 0.75);
        this.draw(r, item);
      }
    }
    this.stale = false;
    this.release(now);
  }

  /**
   * Draws the reflections once in this frame with everything the world draws (no culling, every
   * resident, the glasses and the build helpers too), and the glasses with them, so the pipelines a
   * reflection needs (its target's, its turned faces') are made behind the loading veil, not in the
   * frame a mirror first shows (a pipeline first used in play costs that frame tens of milliseconds;
   * `warmGame` calls it). If no mirror reflects yet, the first ones take the slots. From the game
   * camera's pose (what it sees doesn't matter: nothing is culled). Returns the undo, to run once
   * the frame has drawn. None in a world without mirrors.
   */
  warm(): () => void {
    const forced: Item[] = [];
    const drawn: Reflection[] = [];
    for (let i = 0; i < this.items.length && drawn.length < this.pool.length; i++) {
      const item = this.items[i];
      if (item.slot < 0) {
        let free = -1;
        for (let k = 0; k < this.pool.length && free < 0; k++) if (!this.items.some((x) => x.slot === k)) free = k;
        if (free < 0) break;
        item.slot = free;
        forced.push(item);
      }
      const r = this.reflection(item.slot);
      r.used = performance.now();
      r.viewInv.copy(this.game.entity.getWorldTransform());
      r.proj.copy(this.game.camera.projectionMatrix);
      // (Part of the target, as a glass's rectangle is: the clear is then drawn, and its pipeline made too.)
      r.rect.set(0, 0, 1, 0.5);
      r.camera.rect = r.rect;
      r.camera.scissorRect = r.rect;
      r.camera.frustumCulling = false;
      r.shows.fill(RESIDENT_ROWS_ALL);
      r.showsAnyone = true;
      r.entity.enabled = true;
      for (const d of item.draws) {
        d.material = r.material;
        d.enabled = true;
      }
      drawn.push(r);
    }
    this.warming = drawn.length > 0;
    return () => {
      this.warming = false;
      for (const r of drawn) {
        r.camera.frustumCulling = true;
        r.entity.enabled = false;
        r.owner = null;
        r.dirty = true;
      }
      for (const item of forced) {
        item.slot = -1;
        for (const d of item.draws) d.enabled = false;
      }
      this.changed();
    };
  }

  /** The glasses off (none reflects). */
  private hide(): void {
    if (this.hidden) return;
    this.hidden = true;
    for (const item of this.items) {
      item.slot = -1;
      for (const d of item.draws) if (d.enabled) d.enabled = false;
    }
  }

  /** The game camera's view × projection now (PlayCanvas space, `viewProj`; and game space, `viewProjGame`). */
  private viewProjNow(): void {
    this.view.copy(this.game.entity.getWorldTransform()).invert();
    const vp = this.viewProj.mul2(this.game.camera.projectionMatrix, this.view).data;
    const g = this.viewProjGame;
    for (let k = 0; k < 16; k++) g[k] = vp[k];
    // (Game space to PlayCanvas: z turned round, `space.ts`.)
    for (let k = 8; k < 12; k++) g[k] = -g[k];
  }

  /**
   * Draws `r` for `item` in this frame: its camera's view and projection from the game camera's now,
   * its viewport the glass's rectangle on screen, and the glass's lookup through the view of now.
   */
  private draw(r: Reflection, item: Item): void {
    const c = item.centre;
    const n = item.normal;
    // The glass's plane in PlayCanvas space: n·x + d = 0, the camera's side positive.
    const nx = n[0];
    const ny = n[1];
    const nz = -n[2];
    const d = -(nx * c[0] + ny * c[1] - nz * c[2]);
    const m = this.mirror.data;
    m[0] = 1 - 2 * nx * nx;
    m[1] = -2 * nx * ny;
    m[2] = -2 * nx * nz;
    m[3] = 0;
    m[4] = -2 * nx * ny;
    m[5] = 1 - 2 * ny * ny;
    m[6] = -2 * ny * nz;
    m[7] = 0;
    m[8] = -2 * nx * nz;
    m[9] = -2 * ny * nz;
    m[10] = 1 - 2 * nz * nz;
    m[11] = 0;
    m[12] = -2 * d * nx;
    m[13] = -2 * d * ny;
    m[14] = -2 * d * nz;
    m[15] = 1;
    const viewInv = r.viewInv.mul2(this.mirror, this.game.entity.getWorldTransform());
    const w = viewInv.data;
    // (At the reflected eye: lighting, sorting and the clusters read the camera's position.)
    r.entity.setPosition(w[12], w[13], w[14]);
    const proj = r.proj.copy(this.game.camera.projectionMatrix);
    const P = proj.data;
    // The near plane turned into the glass's (Lengyel's oblique projection): the plane in the
    // reflection camera's view space is Wᵀ·(n, d).
    const cx = nx * w[0] + ny * w[1] + nz * w[2];
    const cy = nx * w[4] + ny * w[5] + nz * w[6];
    const cz = nx * w[8] + ny * w[9] + nz * w[10];
    const cw = nx * w[12] + ny * w[13] + nz * w[14] + d;
    if (cw < 0) {
      const qx = (Math.sign(cx) + P[8]) / P[0];
      const qy = (Math.sign(cy) + P[9]) / P[5];
      const qw = (1 + P[10]) / P[14];
      const s = 2 / (cx * qx + cy * qy - cz + cw * qw);
      P[2] = cx * s;
      P[6] = cy * s;
      P[10] = cz * s + 1;
      P[14] = cw * s;
    }
    // Only the glass's rectangle on screen is drawn (and so only what can be seen through it).
    const tw = r.target.width;
    const th = r.target.height;
    let x0 = 1;
    let y0 = 1;
    let x1 = -1;
    let y1 = -1;
    const vp = this.viewProjGame;
    const k = item.corners;
    let whole = false;
    for (let i = 0; i < 4 && !whole; i++) {
      const clip = this.ndc;
      transform4(vp, k[i * 3], k[i * 3 + 1], k[i * 3 + 2], clip);
      if (clip[3] <= 1e-6) whole = true;
      else {
        const sx = clip[0] / clip[3];
        const sy = clip[1] / clip[3];
        x0 = Math.min(x0, sx);
        x1 = Math.max(x1, sx);
        y0 = Math.min(y0, sy);
        y1 = Math.max(y1, sy);
      }
    }
    if (whole) {
      x0 = y0 = -1;
      x1 = y1 = 1;
    }
    const px0 = Math.max(0, Math.floor(((Math.max(-1, x0) + 1) / 2) * tw) - PAD);
    const px1 = Math.min(tw, Math.ceil(((Math.min(1, x1) + 1) / 2) * tw) + PAD);
    const py0 = Math.max(0, Math.floor(((Math.max(-1, y0) + 1) / 2) * th) - PAD);
    const py1 = Math.min(th, Math.ceil(((Math.min(1, y1) + 1) / 2) * th) + PAD);
    if (px1 <= px0 || py1 <= py0) return;
    r.rect.set(px0 / tw, py0 / th, (px1 - px0) / tw, (py1 - py0) / th);
    r.camera.rect = r.rect;
    r.camera.scissorRect = r.rect;
    // The projection narrowed to that rectangle (the viewport puts it back where it was).
    const X0 = (px0 / tw) * 2 - 1;
    const X1 = (px1 / tw) * 2 - 1;
    const Y0 = (py0 / th) * 2 - 1;
    const Y1 = (py1 / th) * 2 - 1;
    const ax = 2 / (X1 - X0);
    const bx = -(X1 + X0) / (X1 - X0);
    const ay = 2 / (Y1 - Y0);
    const by = -(Y1 + Y0) / (Y1 - Y0);
    for (let j = 0; j < 4; j++) {
      P[j * 4] = ax * P[j * 4] + bx * P[j * 4 + 3];
      P[j * 4 + 1] = ay * P[j * 4 + 1] + by * P[j * 4 + 3];
    }
    const g = this.game;
    const cam = r.camera;
    cam.nearClip = g.nearClip;
    cam.farClip = g.farClip;
    cam.fov = g.fov;
    cam.clearColor = g.clearColor;
    // The glass's lookup: world to clip through the game camera now, to texture coordinates (rows
    // from the bottom on WebGL2, from the top on WebGPU).
    const v = this.viewProj.data;
    const flip = this.device.isWebGPU ? -0.5 : 0.5;
    const L = r.lookup;
    for (let j = 0; j < 4; j++) {
      const w4 = v[j * 4 + 3];
      L[j * 4] = 0.5 * v[j * 4] + 0.5 * w4;
      L[j * 4 + 1] = flip * v[j * 4 + 1] + 0.5 * w4;
      L[j * 4 + 2] = v[j * 4 + 2];
      L[j * 4 + 3] = w4;
    }
    r.drawnViewProj.set(this.viewProjGame);
    r.owner = item;
    r.drawnFace.set(item.face);
    r.dirty = false;
    r.drawing = true;
    this.draws++;
  }

  /**
   * Whether this frame can find anything to draw again: a reflection is due, and it was marked to
   * draw again, or the camera moved since it was drawn, or a resident in its view did.
   */
  private due(now: number, viewProj: Float32Array, residents: MirrorResidents | null): boolean {
    let held = false;
    for (let i = 0; i < this.pool.length; i++) {
      const r = this.pool[i];
      if (!r) continue;
      held = true;
      if (now < r.due - EARLY) continue;
      // Marked to draw again counts only while a glass still shows it: one no glass was given
      // isn't drawn, so its mark would otherwise hold every frame to a full pick until let go.
      if ((r.dirty && r.next?.slot === i) || this.seenMoving(r, viewProj, residents)) return true;
    }
    // No reflection held (none made yet, or all let go): the glasses have to be picked again.
    return !held;
  }

  /** The camera moved since `r` was drawn, or a resident in its view did. */
  private seenMoving(r: Reflection, viewProj: Float32Array, residents: MirrorResidents | null): boolean {
    return !same(viewProj, r.drawnViewProj, STILL) || (r.showsAnyone && !!residents?.posed);
  }

  /** The glass's corners round all its faces, in its first face's plane. */
  private outline(item: Item): void {
    const { surfaces } = item.instance;
    const u = unit(transformNormal(item.face, 1, 0, 0, this.u) as Float32Array);
    const v = unit(transformNormal(item.face, 0, 1, 0, this.v) as Float32Array);
    const c = item.centre;
    let u0 = Infinity;
    let u1 = -Infinity;
    let v0 = Infinity;
    let v1 = -Infinity;
    const p = this.p;
    for (const s of surfaces) {
      multiply(s.matrix, item.matrix, this.world);
      for (const [x, y] of SQUARE) {
        transformPoint(this.world, x, y, 0, p);
        const px = p[0] - c[0];
        const py = p[1] - c[1];
        const pz = p[2] - c[2];
        const a = px * u[0] + py * u[1] + pz * u[2];
        const b = px * v[0] + py * v[1] + pz * v[2];
        u0 = Math.min(u0, a);
        u1 = Math.max(u1, a);
        v0 = Math.min(v0, b);
        v1 = Math.max(v1, b);
      }
    }
    for (let k = 0; k < 4; k++) {
      const [x, y] = SQUARE[k];
      const a = x < 0 ? u0 : u1;
      const b = y < 0 ? v0 : v1;
      for (let j = 0; j < 3; j++) item.corners[k * 3 + j] = c[j] + u[j] * a + v[j] * b;
    }
  }

  /** Whether the glass's corners reach into the camera's view. */
  private onScreen(item: Item): boolean {
    const lo = this.lo.fill(Infinity);
    const hi = this.hi.fill(-Infinity);
    for (let k = 0; k < 4; k++) {
      for (let j = 0; j < 3; j++) {
        lo[j] = Math.min(lo[j], item.corners[k * 3 + j]);
        hi[j] = Math.max(hi[j], item.corners[k * 3 + j]);
      }
    }
    return reaches(this.frustum, 6, lo, hi, 0);
  }

  /** What can be seen through `item`'s glass from the eye: the four planes from the eye's mirror image through its edges, and the glass's own. */
  private view5(item: Item, r: Reflection): void {
    const n = item.normal;
    const c = item.centre;
    const eye = this.eye;
    const image = this.eyeImage;
    const behind = (eye.x - c[0]) * n[0] + (eye.y - c[1]) * n[1] + (eye.z - c[2]) * n[2];
    image[0] = eye.x - 2 * behind * n[0];
    image[1] = eye.y - 2 * behind * n[1];
    image[2] = eye.z - 2 * behind * n[2];
    // A point surely seen: past the glass's middle on the line from the image.
    const ix = 2 * c[0] - image[0];
    const iy = 2 * c[1] - image[1];
    const iz = 2 * c[2] - image[2];
    const P = r.planes;
    const K = item.corners;
    let k = 0;
    for (let i = 0; i < 4; i++) {
      const j = (i + 1) % 4;
      const ax = K[i * 3] - image[0];
      const ay = K[i * 3 + 1] - image[1];
      const az = K[i * 3 + 2] - image[2];
      const bx = K[j * 3] - image[0];
      const by = K[j * 3 + 1] - image[1];
      const bz = K[j * 3 + 2] - image[2];
      let x = ay * bz - az * by;
      let y = az * bx - ax * bz;
      let z = ax * by - ay * bx;
      const length = Math.hypot(x, y, z);
      if (length < 1e-9) continue;
      x /= length;
      y /= length;
      z /= length;
      let dd = -(x * image[0] + y * image[1] + z * image[2]);
      if (x * ix + y * iy + z * iz + dd < 0) {
        x = -x;
        y = -y;
        z = -z;
        dd = -dd;
      }
      P[k * 4] = x;
      P[k * 4 + 1] = y;
      P[k * 4 + 2] = z;
      P[k * 4 + 3] = dd;
      k++;
    }
    P[k * 4] = n[0];
    P[k * 4 + 1] = n[1];
    P[k * 4 + 2] = n[2];
    P[k * 4 + 3] = -(n[0] * c[0] + n[1] * c[1] + n[2] * c[2]);
    r.planeCount = k + 1;
  }

  /** Which residents drawn now stand where the glass shows them (`shows`: the rows for the resident shader). */
  private seesResidents(r: Reflection, residents: MirrorResidents | null): void {
    r.showsAnyone = false;
    r.shows.fill(0);
    if (!residents) return;
    const { heads, visible } = residents;
    const lo = this.lo;
    const hi = this.hi;
    for (let i = 0; i < this.residentCount; i++) {
      if (!visible[i]) continue;
      const x = heads[i * 3];
      const y = heads[i * 3 + 1];
      const z = heads[i * 3 + 2];
      lo[0] = x - RESIDENT_HALF;
      lo[1] = y - RESIDENT_DOWN;
      lo[2] = z - RESIDENT_HALF;
      hi[0] = x + RESIDENT_HALF;
      hi[1] = y + 0.1;
      hi[2] = z + RESIDENT_HALF;
      if (!reaches(r.planes, r.planeCount, lo, hi, MARGIN)) continue;
      r.shows[i >> 4] += 2 ** (i & 15);
      r.showsAnyone = true;
    }
  }

  // --- per camera, while a frame draws --------------------------------------------------------

  /** The reflection drawn by `camera`, if it is a reflection camera. */
  private reflectionOf(camera: pc.CameraComponent | null): Reflection | null {
    if (!camera) return null;
    for (const r of this.pool) if (r?.camera === camera) return r;
    return null;
  }

  /**
   * A reflection camera culls without the glasses and the build helpers, and of the draws never culled
   * (the residents' parts, their contact shadows, quilts, props and ring) only those it may show: a
   * part only with a resident its glass shows, the rest with anyone there.
   */
  private precull(camera: pc.CameraComponent | null): void {
    const r = this.reflectionOf(camera);
    if (!r || !this.worldLayer || this.warming) return;
    const list = this.worldLayer.meshInstances;
    for (let i = 0; i < list.length; i++) {
      const mi = list[i];
      if (!mi.visible) continue;
      let helper = this.helpers.get(mi);
      if (helper === undefined) this.helpers.set(mi, (helper = HELPER.test(mi.node.name)));
      const out = helper || (!mi.cull && !(this.drawsRows(mi, r.shows) ?? r.showsAnyone));
      if (out) {
        mi.visible = false;
        this.hiddenNow.push(mi);
      }
    }
  }

  private postcull(camera: pc.CameraComponent | null): void {
    if (!this.hiddenNow.length || !this.reflectionOf(camera)) return;
    for (const mi of this.hiddenNow) mi.visible = true;
    this.hiddenNow.length = 0;
  }

  /** A reflection camera's layers draw only the residents its glass shows (`game_simShown`). */
  private layerStarts(camera: pc.CameraComponent, _layer: pc.Layer): void {
    const r = this.reflectionOf(camera);
    if (r) this.rowFilter.set(r.shows);
  }

  private layerEnds(camera: pc.CameraComponent, _layer: pc.Layer): void {
    if (this.reflectionOf(camera)) this.rowFilter.fill(RESIDENT_ROWS_ALL);
  }

  // --- reports ---------------------------------------------------------------------------------

  /** How many reflections may be live at once. */
  get capacity(): number {
    return this.pool.length;
  }

  /** Mirrors in the world, and those reflecting now. */
  get count(): number {
    return this.instances.length;
  }

  get live(): number {
    let n = 0;
    for (const item of this.items) if (item.slot >= 0) n++;
    return n;
  }

  /** The reflections' render targets made now. */
  get targets(): pc.RenderTarget[] {
    return this.pool.filter((r): r is Reflection => !!r).map((r) => r.target);
  }

  /** The render targets of the mirrors reflecting now, held ones too (the benchmark's passes). */
  get reflecting(): pc.RenderTarget[] {
    const out: pc.RenderTarget[] = [];
    for (const item of this.items) {
      const r = item.slot >= 0 ? this.pool[item.slot] : null;
      if (r) out.push(r.target);
    }
    return out;
  }

  /** Whether `target` is a reflection's (the benchmark's draw counts, per pass: no allocations). */
  owns(target: pc.RenderTarget | null): boolean {
    if (!target) return false;
    for (const r of this.pool) if (r?.target === target) return true;
    return false;
  }

  /** Where the mirrors are and which way they face (game space; the benchmark's mirror camera). */
  get spots(): { x: number; y: number; z: number; nx: number; ny: number; nz: number }[] {
    const face = new Float32Array(16);
    return this.instances.map(({ matrices, index, surfaces }) => {
      multiply(surfaces[0].matrix, matrices.subarray(index * 16, index * 16 + 16), face);
      const c = transformPoint(face, 0, 0, 0);
      const n = transformNormal(face, 0, 0, 1);
      const l = Math.hypot(n[0], n[1], n[2]) || 1;
      return { x: c[0], y: c[1], z: c[2], nx: n[0] / l, ny: n[1] / l, nz: n[2] / l };
    });
  }

  destroy(): void {
    this.device.off('resizecanvas', this.onResize);
    const scene = this.app.scene;
    scene.off(pc.EVENT_PRECULL, this.onPrecull);
    scene.off(pc.EVENT_POSTCULL, this.onPostcull);
    scene.off(pc.EVENT_PRERENDER_LAYER, this.onLayer);
    scene.off(pc.EVENT_POSTRENDER_LAYER, this.onLayerDone);
    this.rowFilter.fill(RESIDENT_ROWS_ALL);
    this.set([]);
    this.release(Infinity);
    for (const undo of this.unpatch.splice(0)) undo();
    this.idle.destroy();
    this.black.destroy();
    for (const mesh of [this.square, this.disc]) {
      mesh.decRefCount();
      mesh.destroy();
    }
  }
}

/** The glass's material: unlit, its emissive the reflection in `texture` through `lookup` (see the file's header). */
function glassMaterial(texture: pc.Texture, lookup: Float32Array): pc.StandardMaterial {
  const m = new pc.StandardMaterial();
  m.name = 'mirrorGlass';
  m.shaderChunksVersion = '2.8';
  m.useLighting = false;
  m.useSkybox = false;
  m.diffuse.set(0, 0, 0);
  m.specular.set(0, 0, 0);
  m.cull = pc.CULLFACE_NONE;
  m.getShaderChunks(pc.SHADERLANGUAGE_GLSL).set('emissivePS', GLASS_EMISSIVE.glsl);
  m.getShaderChunks(pc.SHADERLANGUAGE_WGSL).set('emissivePS', GLASS_EMISSIVE.wgsl);
  m.setParameter('game_mirror', texture);
  m.setParameter('game_mirrorLookup', lookup);
  m.setParameter('game_mirrorTint', TINT);
  m.update();
  return m;
}

/** The unit square, or the unit-diameter disc, in the xy plane facing +z (game space; its draws place it). */
function unitMesh(device: pc.GraphicsDevice, round: boolean): pc.Mesh {
  const positions: number[] = [];
  const indices: number[] = [];
  if (round) {
    const n = 40;
    positions.push(0, 0, 0);
    for (let i = 0; i <= n; i++) {
      const a = (i / n) * Math.PI * 2;
      positions.push(Math.cos(a) * 0.5, Math.sin(a) * 0.5, 0);
      if (i > 0) indices.push(0, i + 1, i);
    }
  } else {
    positions.push(-0.5, -0.5, 0, 0.5, -0.5, 0, 0.5, 0.5, 0, -0.5, 0.5, 0);
    indices.push(0, 2, 1, 0, 3, 2);
  }
  const mesh = new pc.Mesh(device);
  mesh.setPositions(positions);
  mesh.setNormals(positions.map((_, i) => (i % 3 === 2 ? 1 : 0)));
  mesh.setIndices(indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  // (Held by its draws; the mirrors let go of it last.)
  mesh.incRefCount();
  return mesh;
}

const byDistance = (a: Item, b: Item): number => a.distance - b.distance;

/** The view × projection `m`'s six planes (x, y, z, d each, inside positive; GL clip space). */
function planesOf(m: Float32Array, out: Float32Array): void {
  for (let p = 0; p < 6; p++) {
    const row = p >> 1;
    const sign = p & 1 ? -1 : 1;
    for (let j = 0; j < 4; j++) out[p * 4 + j] = m[j * 4 + 3] + sign * m[j * 4 + row];
  }
}

/** Whether the box `lo`–`hi` reaches into every one of the first `count` planes (inside positive), with `margin` to spare. */
function reaches(planes: Float32Array, count: number, lo: Float32Array, hi: Float32Array, margin: number): boolean {
  for (let p = 0; p < count; p++) {
    const x = planes[p * 4];
    const y = planes[p * 4 + 1];
    const z = planes[p * 4 + 2];
    const far = x * (x >= 0 ? hi[0] : lo[0]) + y * (y >= 0 ? hi[1] : lo[1]) + z * (z >= 0 ? hi[2] : lo[2]) + planes[p * 4 + 3];
    if (far < -margin * Math.hypot(x, y, z)) return false;
  }
  return true;
}

/** Whether two matrices are the same to within `still` (relative). */
function same(a: ArrayLike<number>, b: ArrayLike<number>, still: number): boolean {
  for (let i = 0; i < 16; i++) if (Math.abs(a[i] - b[i]) > still * Math.max(1, Math.abs(a[i]))) return false;
  return true;
}

/** `v` of unit length (in place). */
function unit(v: Float32Array): Float32Array {
  const l = Math.hypot(v[0], v[1], v[2]) || 1;
  v[0] /= l;
  v[1] /= l;
  v[2] /= l;
  return v;
}

/** (x, y, z, 1) through `m` (4 numbers into `out`). */
function transform4(m: Float32Array, x: number, y: number, z: number, out: Float32Array): void {
  for (let j = 0; j < 4; j++) out[j] = m[j] * x + m[4 + j] * y + m[8 + j] * z + m[12 + j];
}

function nearestPOT(x: number): number {
  const c = 2 ** Math.ceil(Math.log2(x));
  const f = 2 ** Math.floor(Math.log2(x));
  return c - x < x - f ? c : f;
}

function floorPOT(x: number): number {
  return 2 ** Math.floor(Math.log2(x));
}
