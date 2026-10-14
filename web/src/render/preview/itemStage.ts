/**
 * The colour studio's stage: one item, live, on a transparent canvas inside a host element, lit
 * like the catalog's pictures (`itemScene.ts`). It turns (drag sideways, with a little momentum;
 * arrow keys), tips (drag up and down) and comes closer (wheel, double-click, +/-). Each new set
 * of colours builds the model again; the one shown stays until the new one is ready.
 */

import { Engine, FreeCamera, Matrix, TransformNode, Vector3, type Mesh, type Scene } from '../babylon/core';
import type { AssetRegistry } from '../../assets/registry';
import { colorsKey, type PartColors } from '../../core/protocol';
import { buildModel, painted } from '../babylon/models';
import type { ItemStage } from '../types';
import { boxCorners, contactShadow, studioScene } from './itemScene';

const FOV = 0.5;
/** Starting view: three-quarter from the front, a little above (radians). */
const START_YAW = -0.62;
const START_PITCH = 0.42;
const MIN_PITCH = 0.02;
const MAX_PITCH = 1.25;
/** Distance as a share of what frames the whole thing (closer and further). */
const MIN_ZOOM = 0.4;
const MAX_ZOOM = 1.5;
/** Give up waiting for a model's materials (ms). */
const READY_TIMEOUT = 8000;

export class ItemStageController implements ItemStage {
  private readonly root: TransformNode;
  private readonly camera: FreeCamera;
  private readonly shadow: Mesh;
  private readonly resize: ResizeObserver;
  private meshes: Mesh[] = [];
  /** Colours wanted next (the latest wins) while a build is under way; undefined: none waiting. */
  private next: { colors: PartColors | undefined } | undefined;
  private building = false;
  private detached = false;
  /** Where the camera looks and how far back it stands to see all of it (set by the first build). */
  private readonly centre = new Vector3(0, 0.5, 0);
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

  private constructor(
    private readonly engine: Engine,
    private readonly scene: Scene,
    private readonly canvas: HTMLCanvasElement,
    host: HTMLElement,
    private readonly assets: AssetRegistry,
    private readonly model: string,
    private readonly footprint: [number, number],
  ) {
    this.root = new TransformNode('stageItem', scene);
    const cam = (this.camera = new FreeCamera('stageItemCamera', new Vector3(0, 2, 5), scene, false));
    cam.fov = FOV;
    cam.minZ = 0.05;
    cam.maxZ = 60;
    cam.inputs.clear();
    scene.activeCamera = cam;
    this.shadow = contactShadow(scene);
    const [fw, fd] = footprint;
    this.shadow.scaling.set(fw * 1.25 + 0.2, 1, fd * 1.25 + 0.2);
    this.shadow.parent = this.root;

    canvas.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;display:block;touch-action:none;cursor:grab;outline:none';
    canvas.setAttribute('aria-label', 'The item: drag or arrow keys to turn, scroll or +/- to come closer');
    canvas.tabIndex = 0;
    host.appendChild(canvas);
    this.resize = new ResizeObserver(() => {
      engine.resize();
      this.restless = 2;
    });
    this.resize.observe(host);
    engine.resize();
    canvas.addEventListener('pointerdown', this.onDown);
    canvas.addEventListener('pointermove', this.onMove);
    canvas.addEventListener('pointerup', this.onUp);
    canvas.addEventListener('pointercancel', this.onUp);
    canvas.addEventListener('wheel', this.onWheel, { passive: false });
    canvas.addEventListener('dblclick', this.onDouble);
    canvas.addEventListener('keydown', this.onKey);
    engine.runRenderLoop(this.tick);
  }

  /** A stage in `host` for `model`; throws when 3D is unavailable. */
  static async create(host: HTMLElement, assets: AssetRegistry, model: string, footprint: [number, number]): Promise<ItemStageController> {
    const canvas = document.createElement('canvas');
    const engine = new Engine(canvas, true, { premultipliedAlpha: true, alpha: true, stencil: false, adaptToDeviceRatio: true });
    try {
      // Sharp enough on a high-density screen without drawing 9 pixels for every one.
      engine.setHardwareScalingLevel(1 / Math.min(2, window.devicePixelRatio || 1));
      const scene = await studioScene(engine, assets);
      return new ItemStageController(engine, scene, canvas, host, assets, model, footprint);
    } catch (err) {
      engine.dispose();
      throw err;
    }
  }

  setColors(colors: PartColors | undefined): void {
    if (this.detached) return;
    this.next = { colors };
    void this.build();
  }

  detach(): void {
    if (this.detached) return;
    this.detached = true;
    this.engine.stopRenderLoop(this.tick);
    this.resize.disconnect();
    this.canvas.remove();
    this.scene.dispose();
    this.engine.dispose();
  }

  /** Builds the latest colours asked for, one build at a time. */
  private async build(): Promise<void> {
    if (this.building) return;
    this.building = true;
    try {
      while (this.next && !this.detached) {
        const { colors } = this.next;
        this.next = undefined;
        const entry = this.assets.has(this.model, 'model') ? this.assets.get(this.model, 'model') : undefined;
        const template = await buildModel(this.scene, `stage:${this.model}|${colorsKey(colors)}`, painted(entry, colors), this.footprint);
        const meshes = template.meshes;
        const identity = Matrix.Identity().asArray() as unknown as Float32Array;
        for (const mesh of meshes) {
          mesh.parent = this.root;
          mesh.isPickable = false;
          mesh.thinInstanceSetBuffer('matrix', identity, 16, true);
          mesh.alwaysSelectAsActiveMesh = true;
          mesh.setEnabled(false);
        }
        const ready = await this.whenReady(meshes);
        // Something newer was asked for meanwhile (or the stage went): this one is never shown.
        if (this.detached || (this.next && ready)) {
          for (const mesh of meshes) mesh.dispose(false, false);
          continue;
        }
        const first = !this.meshes.length;
        for (const mesh of this.meshes) mesh.dispose(false, false);
        this.meshes = meshes;
        for (const mesh of meshes) mesh.setEnabled(true);
        if (first) this.frameModel();
        this.restless = 2;
      }
    } catch (err) {
      console.warn(`[preview] stage of ${this.model} failed`, err);
    } finally {
      this.building = false;
    }
  }

  /** Where to look and how far back to stand: the model's bounds, seen from any side. */
  private frameModel(): void {
    const save = this.root.rotation.y;
    this.root.rotation.y = 0;
    const corners = boxCorners(this.meshes);
    this.root.rotation.y = save;
    if (!corners.length) return;
    const lo = corners[0];
    const hi = corners[corners.length - 1];
    this.centre.set((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, (lo.z + hi.z) / 2);
    const radius = Math.max(0.35, Vector3.Distance(lo, hi) / 2);
    this.reach = (radius / Math.sin(FOV / 2)) * 1.02;
  }

  private async whenReady(meshes: Mesh[]): Promise<boolean> {
    const started = performance.now();
    while (performance.now() - started < READY_TIMEOUT) {
      if (this.detached) return false;
      if (meshes.every((m) => m.isReady(true)) && this.shadow.isReady(true)) return true;
      await new Promise((r) => setTimeout(r, 16));
    }
    return false;
  }

  private readonly tick = () => {
    const now = performance.now() / 1000;
    const dt = this.lastNow ? Math.min(0.1, now - this.lastNow) : 0;
    this.lastNow = now;
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
    this.root.rotation.y = this.yaw;
    const dist = this.reach * this.zoom;
    const c = this.centre;
    this.camera.position.set(c.x, c.y + Math.sin(this.pitch) * dist, c.z + Math.cos(this.pitch) * dist);
    this.camera.setTarget(c);
    this.scene.render();
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
