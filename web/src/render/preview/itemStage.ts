/**
 * The colour studio's stage: one item, live, on a transparent canvas inside a host element, lit
 * like the catalog's pictures (`itemScene.ts`). It turns (drag sideways, with a little momentum;
 * arrow keys), tips (drag up and down) and comes closer (wheel, double-click, +/-). Each new set
 * of colours builds the model again; the one shown stays until the new one is ready.
 *
 * Its clips play as in the game (`poseParts`, as `objectMotion.ts`): the ambient one (`idle`: a fan's
 * blades) all the time, and, while it's open (`setOpen`), those that open (`stageClips`: a fridge's
 * doors, a hot tub's cover) opening and held open, so the parts they hide show, then back to rest,
 * each at its pace in the game.
 */

import * as pc from '../pc/engine';
import type { AssetRegistry } from '../../assets/registry';
import { colorsKey, type PartColors } from '../../core/protocol';
import { buildModel, disposeTemplate, type ModelTemplate } from '../pc/models';
import type { Instances } from '../pc/instancing';
import { poseParts, type PopTarget } from '../pc/objectMotion';
import { toEngine } from '../pc/space';
import { transformPoint } from '../shared/math';
import { partPose, type ModelClip } from '../shared/modelClips';
import { motionsOf, painted, type Motions } from '../shared/modelEntry';
import { restPoses } from '../shared/objectMotion';
import type { ItemStage } from '../types';
import { boxCorners, contactShadow, IDENTITY, PICTURE_STAND_IN, placeModel, shadersCompiling, studioScene, texturesReady, type StudioModel, type StudioScene } from './itemScene';

const FOV = 0.5;
/** Starting view: three-quarter from the front, a little above (radians). */
const START_YAW = -0.62;
const START_PITCH = 0.42;
const MIN_PITCH = 0.02;
const MAX_PITCH = 1.25;
/** Distance as a share of what frames the whole thing (closer and further). */
const MIN_ZOOM = 0.4;
const MAX_ZOOM = 1.5;
/** Stages made (each canvas's id: PlayCanvas keeps its apps by it). */
let stages = 0;

/** A clip the stage plays, and how far it has played (seconds). */
interface StageClip {
  name: string;
  /** Opens while the stage is open and closes again (`use`), or goes round always (`idle`). */
  kind: 'use' | 'idle';
  time: number;
}

export class ItemStageController implements ItemStage {
  /** Turns (`yaw`) about the middle of the model's bounds; `item` holds the model in it, moved so that middle is on its axis. */
  private readonly root: pc.Entity;
  private readonly item: pc.Entity;
  private readonly camera: pc.CameraComponent;
  private readonly resize: ResizeObserver;
  /** The model shown (null before the first is built), and its one instance (in place) for posing its moving parts; null without any. */
  private shown: StudioModel | null = null;
  private template: ModelTemplate | null = null;
  private target: PopTarget | null = null;
  /** Its clips that open (`use`) and its ambient one (`idle`), once the first model is built. */
  private clips: StageClip[] = [];
  private open = false;
  readonly opens: Promise<boolean>;
  private opened!: (opens: boolean) => void;
  /** Colours wanted next (the latest wins) while a build is under way; undefined: none waiting. */
  private next: { colors: PartColors | undefined } | undefined;
  private building = false;
  private detached = false;
  /** Where the camera looks (on the axis it turns about, game space) and how far back it stands to see all of it (set by the first build). */
  private readonly centre = new pc.Vec3(0, 0.5, 0);
  private reach = 2;
  private yaw = START_YAW;
  private yawTarget = START_YAW;
  private yawVel = 0;
  private pitch = START_PITCH;
  private pitchTarget = START_PITCH;
  private zoom = 1;
  private zoomTarget = 1;
  private lastNow = 0;
  /** Frames still to draw before the stage rests (nothing moves, nothing new). */
  private restless = 2;
  private dragging: { id: number; x: number; y: number; t: number } | null = null;
  private frameId = 0;
  private readonly eye = new pc.Vec3();
  private readonly look = new pc.Vec3();

  private constructor(
    private readonly scene: StudioScene,
    private readonly canvas: HTMLCanvasElement,
    private readonly host: HTMLElement,
    private readonly assets: AssetRegistry,
    private readonly model: string,
    private readonly footprint: [number, number],
  ) {
    const app = scene.app;
    this.root = new pc.Entity('stageItem', app);
    scene.root.addChild(this.root);
    // A wall thing hangs at the back of its tile, its origin the tile's middle: turned about that, it would swing out of view.
    this.item = new pc.Entity('stageItemModel', app);
    this.root.addChild(this.item);
    this.opens = new Promise((resolve) => (this.opened = resolve));
    const cam = (this.camera = scene.camera);
    cam.fov = (FOV * 180) / Math.PI;
    cam.nearClip = 0.05;
    cam.farClip = 60;
    const shadow = contactShadow(scene, this.item);
    const [fw, fd] = footprint;
    shadow.setLocalScale(fw * 1.25 + 0.2, 1, fd * 1.25 + 0.2);

    canvas.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;display:block;touch-action:none;cursor:grab;outline:none';
    canvas.setAttribute('aria-label', 'The item: drag or arrow keys to turn, scroll or +/- to come closer');
    canvas.tabIndex = 0;
    host.appendChild(canvas);
    this.resize = new ResizeObserver(() => {
      this.fit();
      this.restless = 2;
    });
    this.resize.observe(host);
    this.fit();
    canvas.addEventListener('pointerdown', this.onDown);
    canvas.addEventListener('pointermove', this.onMove);
    canvas.addEventListener('pointerup', this.onUp);
    canvas.addEventListener('pointercancel', this.onUp);
    canvas.addEventListener('wheel', this.onWheel, { passive: false });
    canvas.addEventListener('dblclick', this.onDouble);
    canvas.addEventListener('keydown', this.onKey);
    this.frameId = requestAnimationFrame(this.tick);
  }

  /** A stage in `host` for `model`; throws when 3D is unavailable. */
  static async create(host: HTMLElement, assets: AssetRegistry, model: string, footprint: [number, number], backend: 'auto' | 'webgl' = 'auto'): Promise<ItemStageController> {
    const canvas = document.createElement('canvas');
    canvas.id = `item-stage-${++stages}`;
    const scene = await studioScene(canvas, assets, { backend, antialias: true, alpha: true });
    try {
      return new ItemStageController(scene, canvas, host, assets, model, footprint);
    } catch (err) {
      scene.destroy();
      throw err;
    }
  }

  setColors(colors: PartColors | undefined): void {
    if (this.detached) return;
    this.next = { colors };
    void this.build();
  }

  setOpen(open: boolean): void {
    this.open = open;
  }

  detach(): void {
    if (this.detached) return;
    this.detached = true;
    this.opened(false);
    cancelAnimationFrame(this.frameId);
    this.resize.disconnect();
    this.canvas.remove();
    // (A build under way lets go of its own model when it sees the stage gone.)
    if (!this.building) this.destroyScene();
  }

  private destroyScene(): void {
    this.shown?.destroy();
    this.shown = null;
    this.scene.destroy();
  }

  /** The canvas's drawing size: the host's, sharp enough on a high-density screen without drawing 9 pixels for every one. */
  private fit(): void {
    const ratio = Math.min(2, window.devicePixelRatio || 1);
    const width = Math.max(1, Math.round(this.host.clientWidth * ratio));
    const height = Math.max(1, Math.round(this.host.clientHeight * ratio));
    if (this.canvas.width === width && this.canvas.height === height) return;
    this.scene.app.setCanvasResolution(pc.RESOLUTION_FIXED, width, height);
  }

  /** Builds the latest colours asked for, one build at a time. */
  private async build(): Promise<void> {
    if (this.building) return;
    this.building = true;
    const { app, lib } = this.scene;
    try {
      while (this.next && !this.detached) {
        const { colors } = this.next;
        this.next = undefined;
        const entry = this.assets.has(this.model, 'model') ? this.assets.get(this.model, 'model') : undefined;
        // Without its file, a plain box (built again with each colour asked for, which tries the file again).
        const template = await buildModel(app, `stage:${this.model}|${colorsKey(colors)}`, painted(entry, colors), this.footprint, { standIn: entry?.url ? PICTURE_STAND_IN : undefined });
        if (this.detached) {
          disposeTemplate(template);
          break;
        }
        const placed = placeModel(this.scene, this.item, template);
        placed.show(false);
        const ready = await texturesReady(lib);
        // Something newer was asked for meanwhile (or the stage went): this one is never shown.
        if (this.detached || (this.next && ready)) {
          placed.destroy();
          continue;
        }
        const first = !this.shown;
        this.shown?.destroy();
        this.shown = placed;
        this.template = template;
        this.target = this.instance(template, placed.draws);
        if (first) this.clips = stageClips(template, motionsOf(entry));
        // Posed where the one before was (half open, say) before it shows.
        if (this.target) poseParts(this.target, this.playing());
        placed.show(true);
        if (first) this.frameModel();
        this.restless = 2;
        this.opened(this.clips.some((c) => c.kind === 'use'));
      }
    } catch (err) {
      console.warn(`[preview] stage of ${this.model} failed`, err);
      this.opened(false);
    } finally {
      this.building = false;
      if (this.detached) this.destroyScene();
    }
  }

  /** The model's one instance, in place, its moving parts drawing from buffers of their own as in the game; null if it has none. */
  private instance(template: ModelTemplate, draws: Instances[]): PopTarget | null {
    const motion = template.motion;
    const count = motion ? Math.max(-1, ...motion.parts) + 1 : 0;
    if (!motion || !count) return null;
    const placed = new Float32Array(IDENTITY);
    const parts = Array.from({ length: count }, (_, k) => ({ meshes: draws.filter((_, i) => motion.parts[i] === k), matrices: placed.slice() }));
    return { meshes: draws.filter((_, i) => motion.parts[i] < 0), matrices: placed, index: 0, moving: { parts, poses: restPoses(count) } };
  }

  /** The clips of the model shown, each where it has played to (`full`: those that open, fully open). */
  private playing(full = false): { clip: ModelClip; time: number; kind: string }[] {
    const clips = this.template?.motion?.clips;
    if (!clips) return [];
    return this.clips.flatMap((c) => {
      const clip = clips.get(c.name);
      if (!clip || (full && c.kind !== 'use')) return [];
      return [{ clip, time: full ? clip.duration : c.time, kind: c.kind }];
    });
  }

  /** Plays the clips on by `dt` seconds, those that open towards where `open` has them, at their pace in the game; true if any moved. */
  private animate(dt: number): boolean {
    const clips = this.template?.motion?.clips;
    if (!clips || !this.target || !dt) return false;
    let moved = false;
    for (const c of this.clips) {
      const d = clips.get(c.name)?.duration ?? 0;
      if (d <= 0) continue;
      const time = c.kind === 'idle' ? (c.time + dt) % d : Math.min(Math.max(c.time + (this.open ? dt : -dt), 0), d);
      if (time === c.time) continue;
      c.time = time;
      moved = true;
    }
    if (moved) poseParts(this.target, this.playing());
    return moved;
  }

  /**
   * Where to look and how far back to stand: the model's bounds, closed and open (a fridge's doors out in
   * front), their middle moved onto the axis it turns about, so it is seen whole from any side.
   */
  private frameModel(): void {
    const meshes = this.template?.meshes ?? [];
    const corners = boxCorners(meshes);
    const lo = corners[0]?.clone();
    const hi = corners[corners.length - 1]?.clone();
    const parts = this.template?.motion?.parts;
    if (lo && hi && parts) {
      const open = this.playing(true);
      const pose = new Float32Array(16);
      const at = [0, 0, 0];
      meshes.forEach((m, i) => {
        const positions: number[] = [];
        if (parts[i] < 0 || !partPose(open, parts[i], pose) || !m.mesh.getPositions(positions)) return;
        // Its vertices posed, not its box: a part grown from nothing (water, flames) lies flat at rest and its
        // pose grows it across the flat many times over, so a box's corners off the flat (a slanted shower's
        // rain, its flat turned) would land metres away, and the camera with them.
        for (let k = 0; k < positions.length; k += 3) {
          transformPoint(pose, positions[k], positions[k + 1], positions[k + 2], at);
          lo.set(Math.min(lo.x, at[0]), Math.min(lo.y, at[1]), Math.min(lo.z, at[2]));
          hi.set(Math.max(hi.x, at[0]), Math.max(hi.y, at[1]), Math.max(hi.z, at[2]));
        }
      });
    }
    if (!lo || !hi) return;
    this.item.setLocalPosition(-(lo.x + hi.x) / 2, 0, -(lo.z + hi.z) / 2);
    this.centre.set(0, (lo.y + hi.y) / 2, 0);
    const radius = Math.max(0.35, lo.distance(hi) / 2);
    this.reach = (radius / Math.sin(FOV / 2)) * 1.02;
  }

  private readonly tick = () => {
    this.frameId = requestAnimationFrame(this.tick);
    const now = performance.now() / 1000;
    const dt = this.lastNow ? Math.min(0.1, now - this.lastNow) : 0;
    this.lastNow = now;
    if (this.animate(dt)) this.restless = 2;
    if (!this.dragging && Math.abs(this.yawVel) > 1e-3) {
      this.yawTarget += this.yawVel * dt;
      this.yawVel *= Math.exp(-dt * 3.5);
    }
    const ease = (from: number, to: number, rate: number) => from + (to - from) * (1 - Math.exp(-dt * rate));
    this.yaw = ease(this.yaw, this.yawTarget, 16);
    this.pitch = ease(this.pitch, this.pitchTarget, 16);
    this.zoom = ease(this.zoom, this.zoomTarget, 8);
    const moving = Math.abs(this.yawTarget - this.yaw) > 1e-4 || Math.abs(this.pitchTarget - this.pitch) > 1e-4 || Math.abs(this.zoomTarget - this.zoom) > 1e-4 || Math.abs(this.yawVel) > 1e-3;
    if (moving) this.restless = 2;
    // At rest, nothing is drawn: the GPU is the game's again.
    if (this.restless <= 0) return;
    this.restless--;
    // (A local turn about y under the world root is Babylon's rotation.y in game space.)
    this.root.setLocalEulerAngles(0, (this.yaw * 180) / Math.PI, 0);
    const dist = this.reach * this.zoom;
    const c = this.centre;
    const entity = this.camera.entity;
    entity.setPosition(toEngine(c.x, c.y + Math.sin(this.pitch) * dist, c.z + Math.cos(this.pitch) * dist, this.eye));
    entity.lookAt(toEngine(c.x, c.y, c.z, this.look));
    const app = this.scene.app;
    app.update(dt);
    app.render();
    // (WebGL2: what is still compiling was left out, `asyncShaders`: drawn again until it's in.)
    if (shadersCompiling(this.scene.device)) this.restless = 2;
  };

  private zoomTo(zoom: number): void {
    this.zoomTarget = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));
  }

  private readonly onDown = (e: PointerEvent) => {
    if (e.button !== 0) return;
    this.dragging = { id: e.pointerId, x: e.clientX, y: e.clientY, t: e.timeStamp };
    this.yawVel = 0;
    this.canvas.setPointerCapture(e.pointerId);
    this.canvas.style.cursor = 'grabbing';
  };

  private readonly onMove = (e: PointerEvent) => {
    const d = this.dragging;
    if (!d || d.id !== e.pointerId) return;
    const dts = Math.max(1, e.timeStamp - d.t) / 1000;
    const turn = (e.clientX - d.x) * 0.011;
    this.yawTarget += turn;
    this.yawVel = this.yawVel * 0.5 + (turn / dts) * 0.5;
    this.pitchTarget = Math.min(MAX_PITCH, Math.max(MIN_PITCH, this.pitchTarget + (e.clientY - d.y) * 0.006));
    d.x = e.clientX;
    d.y = e.clientY;
    d.t = e.timeStamp;
  };

  private readonly onUp = (e: PointerEvent) => {
    if (!this.dragging || this.dragging.id !== e.pointerId) return;
    // A drag that ended with a pause keeps no momentum.
    if (e.timeStamp - this.dragging.t > 80) this.yawVel = 0;
    this.yawVel = Math.max(-5, Math.min(5, this.yawVel));
    this.dragging = null;
    this.canvas.style.cursor = 'grab';
  };

  private readonly onWheel = (e: WheelEvent) => {
    e.preventDefault();
    const step = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY;
    this.zoomTo(this.zoomTarget * Math.exp(step * 0.0015));
  };

  private readonly onDouble = () => this.zoomTo(this.zoomTarget < 0.8 ? 1 : 0.55);

  private readonly onKey = (e: KeyboardEvent) => {
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') this.yawTarget += e.key === 'ArrowLeft' ? -0.35 : 0.35;
    else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') this.pitchTarget = Math.min(MAX_PITCH, Math.max(MIN_PITCH, this.pitchTarget + (e.key === 'ArrowUp' ? 0.15 : -0.15)));
    else if (e.key === '+' || e.key === '=') this.zoomTo(this.zoomTarget * 0.8);
    else if (e.key === '-') this.zoomTo(this.zoomTarget * 1.25);
    else return;
    e.preventDefault();
  };
}

/**
 * The clips the stage plays of `template`: its ambient one (`idle`), and, as those that open, every
 * other clip of its file but those that go round (`loops`, and a use's `every`), the night's, a
 * crop's growth and a clock's hands: what a use or being open plays (a fridge's door, a hot tub's
 * cover) and what nothing plays yet (a fridge's freezer door), as the clips bench does.
 */
function stageClips(template: ModelTemplate, motions: Motions | undefined): StageClip[] {
  const clips = template.motion?.clips;
  if (!clips) return [];
  const uses = Object.values(motions?.uses ?? {}).flat();
  const once = new Set(uses.filter((u) => !u.every).map((u) => u.clip));
  const other = new Set<string | undefined>([...Object.values(motions?.loops ?? {}), motions?.night, motions?.growth, motions?.clock?.hour, motions?.clock?.minute]);
  for (const u of uses) if (u.every && !once.has(u.clip)) other.add(u.clip);
  return [...clips.keys()].flatMap((name): StageClip[] => {
    if (name === motions?.idle) return [{ name, kind: 'idle', time: 0 }];
    return other.has(name) ? [] : [{ name, kind: 'use', time: 0 }];
  });
}
