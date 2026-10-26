/**
 * The preview studio: a small PlayCanvas app of its own (beside the game's, `createSideApp`) running
 * the game's own character pipeline (`Characters`: the same meshes, materials, clips and face rig as
 * in play) under soft studio lights, on a transparent canvas.
 *
 * Sims are driven like in the game, by a snapshot: a synthetic one in which every Sim stands
 * idle, plus stage directions (gestures, expressions, eye contact). Row 0 is the household
 * creator's stage; rows 1.. hold a batch of portraits, standing far off to the side.
 *
 * The app is passed explicitly everywhere (`new pc.Entity(name, app)`): the page's current app,
 * PlayCanvas's default, stays the game's. Everything it caches is its own (per app or device:
 * materials, the sky's lighting, the residents' shaders), and goes with it (`dispose`).
 *
 * The stage's camera (onto the canvas) and the portraits' (into their targets) are drawn by one
 * `app.render()` a frame (`draw`), not a render each.
 */

import * as pc from '../pc/engine';
import { Characters, MAX_CHARACTERS } from '../pc/characters';
import { createDevice, createSideApp, destroySideApp } from '../pc/device';
import { Instances } from '../pc/instancing';
import { ENV_UNITS, SKY_UNITS, SUN_UNITS } from '../pc/lightUnits';
import { contactBlob } from '../pc/lotContents';
import { MaterialLibrary } from '../pc/materials';
import { drawAfter } from '../pc/meshes';
import { pageLook } from '../pc/post';
import type { AssetRegistry } from '../../assets/registry';
import type { FrameState } from '../../core/bridge';
import type { SimInfo } from '../../core/protocol';
import { simFields, type SnapshotLayout } from '../../core/snapshot';
import type { SimLook } from '../types';

export interface StudioDeps {
  assets: AssetRegistry;
  /** Emotion ids in content order (snapshot emotion codes). */
  emotions: readonly string[];
  /** `webgl` forces WebGL2 (as the game's renderer setting does). */
  backend: () => 'auto' | 'webgl';
}

/** A Sim in the studio. `yaw` 0 faces the camera (+z). */
export interface Row {
  look: SimLook;
  x: number;
  z: number;
  yaw: number;
  /** Emotion code (content emotion index + 1; 0 = none) and mood 0..1, as the game sends them. */
  emotion?: number;
  mood?: number;
}

const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood'] as const;
const HEADER_LEN = 6;
const STRIDE = FIELDS.length;
const LAYOUT: SnapshotLayout = {
  capacity: HEADER_LEN + STRIDE * MAX_CHARACTERS,
  headerLen: HEADER_LEN,
  simStride: STRIDE,
  maxSims: MAX_CHARACTERS,
  ticksPerSecond: 20,
  header: { tick: 0, day: 1, minute: 2, speed: 3, simCount: 4, structureVersion: 5 },
  sim: simFields(FIELDS),
  actions: [],
};

/**
 * The studio's image processing, in the materials (ACES at this exposure, then this contrast;
 * `pageLook`).
 */
const EXPOSURE = 1.15;
const CONTRAST = 1.08;
/** The sky's lighting (its strength in the look's units, `ENV_UNITS`). */
const ENVIRONMENT = 0.7;
/** The first portraits wait this long (ms) for the sky's lighting at most (up to 0.3.2 they never waited, the sky being there first). */
const ENV_WAIT = 2000;
/** Drawing size per CSS pixel, at most. */
const MAX_RATIO = 2;
const TRANSPARENT = new pc.Color(0, 0, 0, 0);

export class Studio {
  readonly rows: (Row | null)[] = [];
  /** The floor under the stage Sim: a pool of light and a soft shade (shown with the stage). */
  readonly pool: pc.Entity;
  /** The camera the residents are seen by (the stage's; none: full detail, `Characters`). */
  viewer: pc.CameraComponent | null = null;
  private readonly snap = new Float32Array(LAYOUT.capacity);
  private readonly frame: FrameState;
  private readonly shadows = new Float32Array(MAX_CHARACTERS * 16);
  private readonly blob: Instances;
  private readonly decals: { texture: pc.Texture; material: pc.StandardMaterial }[] = [];
  private builtKey = '';
  private building = false;
  private disposed = false;
  private envSettled = false;
  private readonly born = performance.now();
  private lastDraw = 0;
  private frameId = 0;

  private constructor(
    readonly app: pc.AppBase,
    /** Game space (mirrored in z, as the game's world root: `pc/space.ts`). */
    readonly root: pc.Entity,
    readonly characters: Characters,
    readonly canvas: HTMLCanvasElement,
    env: Promise<boolean> | null,
  ) {
    this.frame = { prev: this.snap, curr: this.snap, alpha: 0, steps: 1, layout: LAYOUT, now: 0 };
    const [blob, pool] = this.makeShadows();
    this.blob = blob;
    this.pool = pool;
    if (env) void env.finally(() => (this.envSettled = true));
    else this.envSettled = true;
  }

  get device(): pc.GraphicsDevice {
    return this.app.graphicsDevice;
  }

  get webgpu(): boolean {
    return this.app.graphicsDevice.isWebGPU;
  }

  /** Creates the app, lights and characters; throws when unavailable. */
  static async create(deps: StudioDeps): Promise<Studio> {
    const canvas = document.createElement('canvas');
    canvas.className = 'sim-studio';
    const forceWebGL = deps.backend() === 'webgl' || new URLSearchParams(location.search).get('renderer') === 'webgl';
    // (Multisampled and transparent: the menus show through.)
    const device = await createDevice(canvas, forceWebGL, true, true);
    const app = createSideApp(canvas, device);
    try {
      const root = new pc.Entity('studio', app);
      root.setLocalScale(1, 1, -1);
      app.root.addChild(root);
      app.scene.ambientLight = new pc.Color(0, 0, 0);
      app.scene.layers.getLayerById(pc.LAYERID_SKYBOX)!.enabled = false;
      const lib = MaterialLibrary.for(app).setAssets(deps.assets);
      const sky = deps.assets.get('environment.sky', 'image');
      // The portraits are lit by the studio lights; the sky is only polish. The first portrait waits
      // for it a moment at most (`ready`), never the menu behind it for a file that may time out.
      const env = sky ? lib.loadEnvironment(sky.url) : null;
      lib.setEnvironmentIntensity(ENVIRONMENT * ENV_UNITS);
      lights(app, lib);
      let studio: Studio | null = null;
      const characters = new Characters(app, root, deps.assets, lib, undefined, { camera: () => studio?.viewer ?? null });
      if (!(await characters.init())) throw new Error('characters unavailable');
      characters.setEmotions(deps.emotions);
      studio = new Studio(app, root, characters, canvas, env);
      return studio;
    } catch (err) {
      destroySideApp(app);
      throw err;
    }
  }

  /** A camera of the studio's, in its look (the image processing in the materials, `pageLook`), off until used. */
  camera(name: string, fov: number, near: number, far: number, priority: number): pc.CameraComponent {
    const entity = new pc.Entity(name, this.app);
    entity.addComponent('camera', { fov: (fov * 180) / Math.PI, nearClip: near, farClip: far, clearColor: TRANSPARENT, priority });
    const camera = entity.camera!;
    pageLook(this.app, camera, EXPOSURE, CONTRAST);
    entity.enabled = false;
    this.app.root.addChild(entity);
    return camera;
  }

  /** Points `camera` from `eye` at `target` (game space). */
  static aim(camera: pc.CameraComponent, eye: { x: number; y: number; z: number }, target: { x: number; y: number; z: number }): void {
    camera.entity.setPosition(eye.x, eye.y, -eye.z);
    camera.entity.lookAt(target.x, target.y, -target.z);
  }

  /** Sizes the canvas's drawing buffer to its box (up to 2 device pixels per CSS pixel). */
  resize(): void {
    const ratio = Math.min(MAX_RATIO, window.devicePixelRatio || 1);
    const width = Math.max(1, Math.round(this.canvas.clientWidth * ratio));
    const height = Math.max(1, Math.round(this.canvas.clientHeight * ratio));
    if (width !== this.device.width || height !== this.device.height) this.app.setCanvasResolution(pc.RESOLUTION_FIXED, width, height);
  }

  /** Calls `frame(now)` every display frame until `stop`. */
  run(frame: (now: number) => void): void {
    cancelAnimationFrame(this.frameId);
    const tick = (now: number) => {
      this.frameId = requestAnimationFrame(tick);
      frame(now);
    };
    this.frameId = requestAnimationFrame(tick);
  }

  stop(): void {
    cancelAnimationFrame(this.frameId);
    this.frameId = 0;
  }

  /** Draws every enabled camera (the stage's onto the canvas, the portraits' into their targets). */
  draw(now: number): void {
    const dt = this.lastDraw ? Math.min(0.1, (now - this.lastDraw) / 1000) : 0;
    this.lastDraw = now;
    this.app.update(dt);
    this.app.render();
  }

  /** Puts a Sim into row `i` (null empties it). New looks are rebuilt before the next pose. */
  setRow(i: number, row: Row | null): void {
    if (row) this.rows[i] = row;
    else if (i < this.rows.length) this.rows[i] = null;
    while (this.rows.length && !this.rows[this.rows.length - 1]) this.rows.length--;
    this.sync();
  }

  /** Whether every row's current look is built (its draws exist; their shaders compile when first drawn). */
  get synced(): boolean {
    return !this.building && this.builtKey === this.structureKey();
  }

  /**
   * Whether the Sims can be drawn as they should look: built, and the sky's lighting there (or given
   * up on after a moment). The residents' materials have no textures, and PlayCanvas compiles a
   * shader when it first draws with it, so both are waited for.
   */
  ready(): boolean {
    return this.synced && (this.envSettled || performance.now() - this.born > ENV_WAIT);
  }

  /** Poses every Sim for this frame (same code path as the game), and their contact shadows. */
  pose(now: number): void {
    const s = this.snap;
    const h = LAYOUT.header;
    const k = LAYOUT.sim;
    s[h.speed] = 1;
    s[h.simCount] = this.rows.length;
    for (let i = 0; i < this.rows.length; i++) {
      const row = this.rows[i];
      const o = HEADER_LEN + i * STRIDE;
      s[o + k.id] = i;
      s[o + k.x] = row ? row.x : 0;
      s[o + k.z] = row ? row.z : 0;
      s[o + k.yaw] = row ? row.yaw : 0;
      s[o + k.pose] = 0;
      s[o + k.moving] = 0;
      s[o + k.social] = 0;
      s[o + k.outcome] = 0;
      s[o + k.partner] = -1;
      s[o + k.anim] = 0;
      s[o + k.emotion] = row?.emotion ?? 0;
      s[o + k.role] = 0;
      s[o + k.away] = row ? 0 : 1;
      s[o + k.object] = -1;
      s[o + k.action] = -1;
      s[o + k.mood!] = row?.mood ?? 0.8;
    }
    this.frame.now = now;
    this.characters.update(this.frame, always, this.shadows);
    const blob = this.blob;
    const count = this.rows.length;
    blob.setCount(count);
    for (let i = 0; i < count; i++) blob.setMatrix(i, this.shadows, i * 16);
    blob.update(false);
    blob.enabled = count > 0;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.stop();
    this.characters.dispose();
    this.blob.destroy();
    for (const d of this.decals) {
      d.material.destroy();
      d.texture.destroy();
    }
    destroySideApp(this.app);
    this.canvas.remove();
  }

  private structureKey(): string {
    return JSON.stringify(this.rows.map((r) => (r ? [r.look.gender, r.look.appearance, r.look.id ?? null] : null)));
  }

  private sync(): void {
    const key = this.structureKey();
    if (this.building || key === this.builtKey || this.disposed) return;
    this.building = true;
    const sims = this.rows.flatMap((r, i) => (r ? [simInfo(i, r.look)] : []));
    void this.characters
      .build({ sims, objects: [] }, 'rows')
      .catch((err: unknown) => console.warn('[preview] build failed', err))
      .finally(() => {
        this.builtKey = key;
        this.building = false;
        // Rows may have changed meanwhile.
        this.sync();
      });
  }

  /**
   * Contact shadows (the game's soft blob, per Sim) and, for the stage, a round frosted platform
   * with a soft shade falling away from the key light, so the Sim stands on something over
   * whatever the menus show behind the canvas.
   */
  private makeShadows(): [Instances, pc.Entity] {
    const app = this.app;
    const square = contactBlob(app).mesh;
    /** An unlit decal: `color` (sRGB) through the alpha `draw` paints, drawn in `order` (its `alphaIndex`). */
    const decal = (name: string, draw: (ctx: CanvasRenderingContext2D, n: number) => void, color: pc.Color, order: number) => {
      const n = 256;
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = n;
      const ctx = canvas.getContext('2d')!;
      draw(ctx, n);
      const texture = new pc.Texture(app.graphicsDevice, { name, width: n, height: n, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
      texture.setSource(canvas);
      const material = new pc.StandardMaterial();
      material.name = name;
      material.useLighting = false;
      material.useSkybox = false;
      material.useFog = false;
      material.diffuse.set(0, 0, 0);
      material.emissive = color;
      material.opacityMap = texture;
      material.opacityMapChannel = 'a';
      // The colour blended over, the alphas added. On the transparent
      // canvas a shade over the platform hides more of the page behind it.
      material.blendState = new pc.BlendState(true, pc.BLENDEQUATION_ADD, pc.BLENDMODE_SRC_ALPHA, pc.BLENDMODE_ONE_MINUS_SRC_ALPHA, pc.BLENDEQUATION_ADD, pc.BLENDMODE_ONE, pc.BLENDMODE_ONE);
      material.depthWrite = false;
      material.cull = pc.CULLFACE_NONE;
      material.update();
      this.decals.push({ texture, material });
      return { material, order };
    };
    const radial = (stops: [number, number][]) => (ctx: CanvasRenderingContext2D, n: number) => {
      const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
      for (const [at, a] of stops) g.addColorStop(at, `rgba(0,0,0,${a})`);
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, n, n);
    };
    const black = new pc.Color(0, 0, 0);
    const blobLook = decal(
      'studioBlob',
      radial([
        [0, 0.6],
        [0.5, 0.32],
        [1, 0],
      ]),
      black,
      3,
    );
    const blob = new Instances(app.graphicsDevice, this.root, square, blobLook.material, { name: 'studioBlob', castShadows: false, receiveShadows: false, capacity: MAX_CHARACTERS, cull: false });
    drawAfter(blob.meshInstance, blobLook.order);
    blob.enabled = false;

    const floor = new pc.Entity('studioFloor', app);
    this.root.addChild(floor);
    const place = (name: string, look: { material: pc.Material; order: number }, size: number) => {
      const entity = new pc.Entity(name, app);
      const mi = new pc.MeshInstance(square, look.material);
      drawAfter(mi, look.order);
      entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false });
      entity.setLocalScale(size, 1, size);
      floor.addChild(entity);
      return entity;
    };
    // The platform: a frosted disc, brighter towards its rim, with a thin highlight edge.
    const disc = place(
      'studioDisc',
      decal(
        'studioDisc',
        (ctx, n) => {
          const r = n / 2 - 3;
          const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, r);
          g.addColorStop(0, 'rgba(0,0,0,0.42)');
          g.addColorStop(0.82, 'rgba(0,0,0,0.5)');
          g.addColorStop(1, 'rgba(0,0,0,0.62)');
          ctx.fillStyle = g;
          ctx.beginPath();
          ctx.arc(n / 2, n / 2, r, 0, Math.PI * 2);
          ctx.fill();
          ctx.lineWidth = 3;
          ctx.strokeStyle = 'rgba(0,0,0,0.95)';
          ctx.stroke();
        },
        new pc.Color(0.97, 0.96, 0.94),
        0,
      ),
      1.5,
    );
    disc.setLocalPosition(0, 0.002, 0);
    // The Sim's shade on the platform, falling away from the key light (behind, to the right).
    const shade = place(
      'studioShade',
      decal(
        'studioShade',
        radial([
          [0, 0.24],
          [0.5, 0.12],
          [1, 0],
        ]),
        black,
        1,
      ),
      1,
    );
    shade.setLocalScale(0.95, 1, 1.5);
    // (A turn about y in game space: the same matrix under the mirrored root.)
    shade.setLocalEulerAngles(0, (-0.65 * 180) / Math.PI, 0);
    shade.setLocalPosition(-0.22, 0.004, -0.3);
    floor.enabled = false;
    return [blob, floor];
  }
}

const always = () => true;

function simInfo(id: number, look: SimLook): SimInfo & { outfitId: number } {
  return {
    id,
    name: '',
    household: 0,
    gender: look.gender,
    attractedTo: [],
    appearance: look.appearance,
    traits: [],
    perks: [],
    outfitId: look.id ?? 0,
  };
}

/**
 * Soft studio lighting, fixed in the world (the stage turns the Sim, not the camera, so portraits
 * and the stage share it): a warm key from the upper left of the view, a cool fill from the right,
 * two rims from behind, and the game's sky environment for ambient and reflections.
 * (The camera looks down -z, so the view's left is +x.)
 *
 * The lights' strengths are in the look's units, converted as the dev pages convert them
 * (`lightUnits.ts`, `debug/pcStage.ts`): directional lights at `SUN_UNITS`, their linear colour and
 * strength as PlayCanvas takes them (sRGB, the strength in the colour); the hemispheric light as the
 * library's sky fill (`SKY_UNITS`). The fill had a faint highlight up to 0.3.2 (specular 0.15 of its
 * colour): here none (PlayCanvas's lights have no specular colour of their own).
 */
function lights(app: pc.AppBase, lib: MaterialLibrary): void {
  const directional = (name: string, dir: [number, number, number], intensity: number, color: [number, number, number], specular = true) => {
    const gamma = (c: number) => Math.pow(c * intensity * SUN_UNITS, 1 / 2.2);
    const entity = new pc.Entity(name, app);
    entity.addComponent('light', { type: 'directional', castShadows: false, intensity: 1, color: new pc.Color(gamma(color[0]), gamma(color[1]), gamma(color[2])), affectSpecularity: specular });
    // A directional light shines along its entity's -y: turned from straight down to the light's way (game space to PlayCanvas's).
    const to = new pc.Vec3(dir[0], dir[1], -dir[2]).normalize();
    const down = new pc.Vec3(0, -1, 0);
    const axis = new pc.Vec3().cross(down, to);
    const angle = (Math.acos(Math.max(-1, Math.min(1, down.dot(to)))) * 180) / Math.PI;
    entity.setRotation(axis.length() > 1e-6 ? new pc.Quat().setFromAxisAngle(axis.normalize(), angle) : pc.Quat.IDENTITY);
    app.root.addChild(entity);
  };
  directional('studioKey', [-0.5, -0.58, -0.64], 2.9, [1, 0.95, 0.88]);
  directional('studioFill', [0.78, -0.2, -0.6], 0.95, [0.84, 0.9, 1], false);
  directional('studioRim', [0.45, -0.4, 0.8], 2.8, [1, 0.96, 0.92]);
  directional('studioRim2', [-0.6, -0.3, 0.75], 1.5, [0.86, 0.92, 1]);
  lib.setSkyFill({ r: 0.95, g: 0.97, b: 1 }, { r: 0.45, g: 0.4, b: 0.38 }, 0.35 * SKY_UNITS);
  lib.setSkyDirection({ x: 0, y: 1, z: 0 });
}
