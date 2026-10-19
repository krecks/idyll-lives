/**
 * Mirror reflections. Every mirror's glass (a model's `mirrors`) gets a flat stand-in just in front
 * of it; the few nearest the camera that are on screen and face it show the room in a
 * `MirrorTexture` (the scene drawn once more, reflected in the glass's plane, at a share of the
 * screen's size). The rest keep their glossy glass. A reflection is only drawn while its mirror is
 * on screen, and draws only what can be seen through the glass (the pyramid from the reflected eye
 * through the mirror's edges), not the whole reflected view.
 *
 * With a `rate` (Low and Medium) a reflection is drawn on demand and held in between: again when
 * the camera moves, the mirror moves (a cabinet's door), a resident in its view moves, or the
 * renderer says something else changed (`changed`: the light, lamps, things placed), at most
 * `rate` times a second. A held reflection is looked up through the camera it was drawn for, so
 * it stays on the glass while the camera moves on (only its parallax lags, until the next draw).
 * Without one (High) every reflection is drawn every frame.
 *
 * Mirrors follow their object's instance matrix each frame, so things popping in, hidden storeys
 * and objects being moved need nothing extra. A reflection's render target exists only while a
 * mirror uses it (and a little after, as mirrors come and go with the camera's turns).
 */

import { Color3, Constants, Frustum, Matrix, Mesh, MirrorTexture, Plane, StandardMaterial, Vector3, VertexData, type AbstractMesh, type Camera, type Scene } from './core';
import type { MirrorSurface } from './models';

/** A placed object with mirror glass. */
export interface MirrorInstance {
  surfaces: MirrorSurface[];
  /** Its template's instance matrices (16 floats each) and its index there. */
  matrices: Float32Array;
  index: number;
}

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

/**
 * Residents as the characters last posed them: head tops (x, y, z per row), which are drawn, how
 * many rows, whether any moved, and the rows a mesh of theirs draws (null: not a resident's).
 */
export interface MirrorResidents {
  heads: Float32Array;
  visible: Uint8Array;
  count: number;
  moved: boolean;
  rowsOf(mesh: AbstractMesh): { rows: Float32Array; count: number } | null;
}

/** Mirrors further from the camera than this (m) aren't worth a reflection. */
const RANGE = 40;
/** Milliseconds a reflection's render target is kept once no mirror uses it. */
const KEEP_MS = 10_000;
/** Metres a mesh's bounds may lie outside a mirror's view and still be drawn (the eye moves on a little before the draw). */
const MARGIN = 0.3;
/** A resident's box round their head top: half its width, and how far it reaches down (m). */
const RESIDENT_HALF = 0.4;
const RESIDENT_DOWN = 1.9;
/** Milliseconds a draw may come before it is due (frames don't land on the time exactly). */
const EARLY = 4;
/** The camera counts as moved past this change in any of its view × projection's numbers. */
const STILL = 1e-5;
/** Texture coordinates from clip space, as Babylon's projection mode has them. */
const CLIP_TO_UV = Matrix.FromValues(0.5, 0, 0, 0, 0, -0.5, 0, 0, 0, 0, 0, 0, 0.5, 0.5, 1, 1);

interface Reflection {
  texture: MirrorTexture;
  material: StandardMaterial;
  /** When a mirror last used it (`performance.now()`). */
  used: number;
  /** The mirror it shows, and its face and the camera's view × projection when last drawn. */
  owner: Item | null;
  drawnFace: Matrix;
  drawnViewProj: Matrix;
  /** When the next draw is due (`performance.now()`), held to `rate`. */
  due: number;
  /** Who it is being drawn for now, and something in it changed since it was drawn. */
  next: Item | null;
  dirty: boolean;
  /** What can be seen through the glass: the planes a mesh's bounds must reach into (inside positive). */
  planes: Plane[];
  planeCount: number;
  /** The draw list made from those (reused). */
  list: AbstractMesh[];
  /** Per Sim row: they stand where the glass shows them; and whether anyone does. */
  shows: Uint8Array;
  showsAnyone: boolean;
  /** The texture lookup for the camera now (reused). */
  lookup: Matrix;
}

interface Item {
  instance: MirrorInstance;
  meshes: Mesh[];
  /** Per mesh: its world matrix (Babylon keeps the one a mesh is frozen with). */
  worlds: Matrix[];
  /** Pool slot showing its reflection, or -1. */
  slot: number;
  /** Its object's instance matrix this frame, and its first face's world matrix. */
  matrix: Matrix;
  face: Matrix;
  centre: Vector3;
  normal: Vector3;
  /** The glass's corners round all its faces, in the first face's plane (world). */
  corners: Vector3[];
  distance: number;
}

export class Mirrors {
  /** The reflections that may be live at once (null: not made, or let go). */
  private readonly pool: (Reflection | null)[];
  private readonly square: Mesh;
  private readonly disc: Mesh;
  private readonly overlays = new Set<AbstractMesh>();
  private items: Item[] = [];
  /** The world's mirrors, reflecting or not. */
  private instances: MirrorInstance[] = [];
  private readonly world = new Matrix();
  private readonly back = new Vector3();
  private readonly ranked: Item[] = [];
  private readonly view = Array.from({ length: 6 }, () => new Plane(0, 0, 0, 0));
  private readonly eyeImage = new Vector3();
  private readonly tmp = [new Vector3(), new Vector3(), new Vector3(), new Vector3()];
  private readonly lo = new Vector3();
  private readonly hi = new Vector3();
  private readonly inverse = new Matrix();
  /** Something the reflections show changed (see `changed`). */
  private stale = false;
  /** The residents as last posed (for the draw lists). */
  private residents: MirrorResidents | null = null;
  private readonly resize: () => void;
  /** Reflections drawn so far (the benchmark's count). */
  draws = 0;

  /** A reflection made (the benchmark watches what each draws). */
  onCreated: ((texture: MirrorTexture) => void) | null = null;

  /** `reflected`: whether a mesh shows in reflections (the overlays never do). */
  constructor(
    private readonly scene: Scene,
    private readonly quality: MirrorQuality,
    private readonly reflected: (mesh: AbstractMesh) => boolean,
  ) {
    this.square = unitMesh(scene, 'mirrorSquare', false);
    this.disc = unitMesh(scene, 'mirrorDisc', true);
    this.pool = Array.from({ length: quality.count }, () => null);
    // A resized reflection starts empty.
    this.resize = () => this.changed();
    scene.getEngine().onResizeObservable.add(this.resize);
  }

  private reflection(i: number): Reflection {
    const known = this.pool[i];
    if (known) return known;
    const scene = this.scene;
    const texture = new MirrorTexture(`mirror${i}`, { ratio: this.quality.scale }, scene, false, Constants.TEXTURETYPE_HALF_FLOAT);
    const r: Reflection = {
      texture,
      material: new StandardMaterial(`mirror${i}`, scene),
      used: 0,
      owner: null,
      drawnFace: new Matrix(),
      drawnViewProj: new Matrix(),
      due: -Infinity,
      next: null,
      dirty: true,
      planes: Array.from({ length: 5 }, () => new Plane(0, 0, 0, 0)),
      planeCount: 0,
      list: [],
      shows: new Uint8Array(0),
      showsAnyone: false,
      lookup: new Matrix(),
    };
    texture.renderListPredicate = (mesh) => !this.overlays.has(mesh) && this.reflected(mesh);
    texture.getCustomRenderList = (_pass, meshes, count) => this.cull(r, meshes, count);
    // Unsmoothed edges show as stairs at this size (Low leaves them).
    texture.samples = this.quality.samples;
    texture.refreshRate = this.quality.rate > 0 ? MirrorTexture.REFRESHRATE_RENDER_ONCE : 1;
    // What it was drawn for: the camera's view × projection (before the mirror texture turns the scene's view round).
    texture.onBeforeRenderObservable.add(
      () => {
        r.drawnViewProj.copyFrom(scene.getTransformMatrix());
        r.owner = r.next;
        if (r.owner) r.drawnFace.copyFrom(r.owner.face);
        r.dirty = false;
        this.draws++;
      },
      undefined,
      true,
    );
    // Looked up through the camera it was drawn for: clip space then, from the view now.
    texture.getReflectionTextureMatrix = () => {
      scene.getViewMatrix().invertToRef(this.inverse);
      this.inverse.multiplyToRef(r.drawnViewProj, r.lookup);
      return r.lookup.multiplyToRef(CLIP_TO_UV, r.lookup);
    };
    // A hint of the glass's own tint and loss.
    texture.level = 0.92;
    const material = r.material;
    material.diffuseColor = Color3.Black();
    material.specularColor = Color3.Black();
    material.emissiveColor = new Color3(0.006, 0.008, 0.009);
    material.disableLighting = true;
    material.reflectionTexture = texture;
    this.onCreated?.(texture);
    return (this.pool[i] = r);
  }

  /** Lets go of the reflections no mirror has used for `KEEP_MS` (all of them with `now` Infinity). */
  private release(now: number): void {
    this.pool.forEach((r, i) => {
      if (!r || now - r.used < KEEP_MS || this.items.some((item) => item.slot === i)) return;
      for (const item of this.items) for (const mesh of item.meshes) if (mesh.material === r.material) mesh.material = null;
      r.material.dispose();
      r.texture.dispose();
      this.pool[i] = null;
    });
  }

  /** The mirrors of the world being shown (replaces the last ones). */
  set(instances: MirrorInstance[]): void {
    for (const item of this.items) for (const mesh of item.meshes) mesh.dispose();
    this.overlays.clear();
    this.items = [];
    this.instances = instances.filter((i) => i.surfaces.length);
    for (const r of this.pool) if (r) r.owner = r.next = null;
    // A world without mirrors needs none of the reflections.
    if (!instances.some((i) => i.surfaces.length)) this.release(Infinity);
    if (!this.pool.length) return;
    for (const instance of instances) {
      if (!instance.surfaces.length) continue;
      const meshes = instance.surfaces.map((s, i) => {
        const mesh = (s.round ? this.disc : this.square).clone(`mirror:${instance.index}:${i}`);
        mesh.setEnabled(false);
        mesh.isPickable = false;
        this.overlays.add(mesh);
        return mesh;
      });
      const corners = Array.from({ length: 4 }, () => new Vector3());
      this.items.push({ instance, meshes, worlds: meshes.map(() => new Matrix()), slot: -1, matrix: new Matrix(), face: new Matrix(), centre: new Vector3(), normal: new Vector3(), corners, distance: 0 });
    }
    this.changed();
  }

  /** Something the reflections show changed (the light, lamps, things placed or moving): held ones are drawn again. */
  changed(): void {
    this.stale = true;
  }

  /**
   * Per frame: places the mirrors, gives the reflections to the nearest ones on screen facing
   * `camera`, and says which held ones to draw again. `residents`: as just posed.
   */
  update(camera: Camera, residents: MirrorResidents | null): void {
    this.residents = residents;
    if (!this.items.length) {
      this.stale = false;
      return;
    }
    const now = performance.now();
    const eye = camera.globalPosition;
    Frustum.GetPlanesToRef(camera.getTransformationMatrix(), this.view);
    const ranked = this.ranked;
    ranked.length = 0;
    for (const item of this.items) {
      const { matrices, index, surfaces } = item.instance;
      Matrix.FromArrayToRef(matrices, index * 16, item.matrix);
      surfaces[0].matrix.multiplyToRef(item.matrix, item.face);
      Vector3.TransformCoordinatesFromFloatsToRef(0, 0, 0, item.face, item.centre);
      Vector3.TransformNormalFromFloatsToRef(0, 0, 1, item.face, item.normal);
      const length = item.normal.length();
      item.distance = Vector3.Distance(eye, item.centre);
      // Hidden (scaled away), turned away from the camera, too far to matter, or off screen.
      if (length < 1e-6) continue;
      item.normal.scaleInPlace(1 / length);
      const facing = (eye.x - item.centre.x) * item.normal.x + (eye.y - item.centre.y) * item.normal.y + (eye.z - item.centre.z) * item.normal.z;
      if (facing <= 0 || item.distance >= RANGE) continue;
      this.outline(item);
      if (this.onScreen(item)) ranked.push(item);
    }
    ranked.sort((a, b) => a.distance - b.distance);
    const chosen = new Set(ranked.slice(0, this.pool.length));
    // Keep each chosen mirror's slot where it can (no flicker as the order changes).
    const free = new Set(this.pool.keys());
    for (const item of this.items) {
      if (chosen.has(item) && item.slot >= 0) free.delete(item.slot);
      else item.slot = -1;
    }
    const period = this.quality.rate > 0 ? 1000 / this.quality.rate : 0;
    const viewProj = camera.getTransformationMatrix();
    for (const item of this.items) {
      if (chosen.has(item) && item.slot < 0) {
        item.slot = free.values().next().value!;
        free.delete(item.slot);
      }
      const on = item.slot >= 0;
      let r: Reflection | null = null;
      if (on) {
        r = this.reflection(item.slot);
        r.used = now;
        r.next = item;
        this.view5(item, eye, r);
        this.seesResidents(r, residents);
      }
      item.meshes.forEach((mesh, i) => {
        if (mesh.isEnabled() !== on) mesh.setEnabled(on);
        if (!r) return;
        if (mesh.material !== r.material) mesh.material = r.material;
        item.instance.surfaces[i].matrix.multiplyToRef(item.matrix, item.worlds[i]);
        mesh.freezeWorldMatrix(item.worlds[i]);
        // The plane faces into the glass: what is behind it is clipped from the reflection.
        if (i === 0) Plane.FromPositionAndNormalToRef(item.centre, item.normal.negateToRef(this.back), r.texture.mirrorPlane);
      });
      if (!r || !period) continue;
      // Held: drawn again at once for another mirror; when something in it changed, once it's due.
      if (r.owner !== item) {
        r.texture.resetRefreshCounter();
        continue;
      }
      if (this.stale || !same(item.face, r.drawnFace) || !same(viewProj, r.drawnViewProj) || (r.showsAnyone && residents?.moved)) r.dirty = true;
      if (r.dirty && now >= r.due - EARLY) {
        r.texture.resetRefreshCounter();
        // Steady at `rate` while it keeps changing; after a still spell, not two draws in a row.
        r.due = Math.max(r.due + period, now + period * 0.75);
      }
    }
    this.stale = false;
    this.release(now);
  }

  /** The glass's corners round all its faces, in its first face's plane. */
  private outline(item: Item): void {
    const { surfaces } = item.instance;
    const u = Vector3.TransformNormalFromFloatsToRef(1, 0, 0, item.face, this.tmp[0]).normalize();
    const v = Vector3.TransformNormalFromFloatsToRef(0, 1, 0, item.face, this.tmp[1]).normalize();
    let u0 = Infinity;
    let u1 = -Infinity;
    let v0 = Infinity;
    let v1 = -Infinity;
    const p = this.tmp[2];
    for (const s of surfaces) {
      s.matrix.multiplyToRef(item.matrix, this.world);
      for (const [x, y] of SQUARE) {
        Vector3.TransformCoordinatesFromFloatsToRef(x, y, 0, this.world, p).subtractInPlace(item.centre);
        const a = Vector3.Dot(p, u);
        const b = Vector3.Dot(p, v);
        u0 = Math.min(u0, a);
        u1 = Math.max(u1, a);
        v0 = Math.min(v0, b);
        v1 = Math.max(v1, b);
      }
    }
    SQUARE.forEach(([x, y], k) => {
      const corner = item.corners[k];
      corner.copyFrom(item.centre);
      corner.addInPlace(u.scale(x < 0 ? u0 : u1)).addInPlace(v.scale(y < 0 ? v0 : v1));
    });
  }

  /** Whether the glass's corners reach into the camera's view. */
  private onScreen(item: Item): boolean {
    this.lo.setAll(Infinity);
    this.hi.setAll(-Infinity);
    for (const c of item.corners) {
      this.lo.minimizeInPlace(c);
      this.hi.maximizeInPlace(c);
    }
    return reaches(this.view, 6, this.lo, this.hi, 0);
  }

  /** What can be seen through `item`'s glass from `eye`: the four planes from the eye's mirror image through its edges, and the glass's own. */
  private view5(item: Item, eye: Vector3, r: Reflection): void {
    const n = item.normal;
    const image = this.eyeImage;
    const behind = (eye.x - item.centre.x) * n.x + (eye.y - item.centre.y) * n.y + (eye.z - item.centre.z) * n.z;
    image.set(eye.x - 2 * behind * n.x, eye.y - 2 * behind * n.y, eye.z - 2 * behind * n.z);
    // A point surely seen: past the glass's middle on the line from the image.
    const inside = this.tmp[3].copyFrom(item.centre).scaleInPlace(2).subtractInPlace(image);
    let k = 0;
    for (let i = 0; i < 4; i++) {
      const a = item.corners[i].subtract(image);
      const b = item.corners[(i + 1) % 4].subtract(image);
      const normal = Vector3.Cross(a, b);
      const length = normal.length();
      if (length < 1e-9) continue;
      normal.scaleInPlace(1 / length);
      const plane = r.planes[k++];
      plane.normal.copyFrom(normal);
      plane.d = -Vector3.Dot(normal, image);
      if (plane.signedDistanceTo(inside) < 0) {
        plane.normal.scaleInPlace(-1);
        plane.d = -plane.d;
      }
    }
    const glass = r.planes[k++];
    glass.normal.copyFrom(n);
    glass.d = -Vector3.Dot(n, item.centre);
    r.planeCount = k;
  }

  /**
   * The meshes of the list that reach into what the glass shows. Meshes the camera never culls
   * (residents: posed on the GPU, their bounds say nothing; their shadows, quilts and markers) only
   * with a resident they draw where the glass shows them, and the rest only with anyone there.
   */
  private cull(r: Reflection, meshes: readonly AbstractMesh[] | null, count: number): AbstractMesh[] | null {
    if (!meshes) return null;
    const list = r.list;
    list.length = 0;
    for (let i = 0; i < count; i++) {
      const mesh = meshes[i];
      if (mesh.alwaysSelectAsActiveMesh) {
        if (!r.showsAnyone) continue;
        const drawn = this.residents?.rowsOf(mesh);
        if (!drawn) list.push(mesh);
        else
          for (let j = 0; j < drawn.count; j++)
            if (r.shows[drawn.rows[j * 4]]) {
              list.push(mesh);
              break;
            }
      } else {
        const box = mesh.getBoundingInfo().boundingBox;
        if (reaches(r.planes, r.planeCount, box.minimumWorld, box.maximumWorld, MARGIN)) list.push(mesh);
      }
    }
    return list;
  }

  /** Which residents drawn now stand where the glass shows them. */
  private seesResidents(r: Reflection, residents: MirrorResidents | null): void {
    r.showsAnyone = false;
    if (!residents) return;
    const { heads, visible, count } = residents;
    if (r.shows.length < count) r.shows = new Uint8Array(heads.length / 3);
    r.shows.fill(0);
    for (let i = 0; i < count; i++) {
      if (!visible[i]) continue;
      const x = heads[i * 3];
      const y = heads[i * 3 + 1];
      const z = heads[i * 3 + 2];
      this.lo.set(x - RESIDENT_HALF, y - RESIDENT_DOWN, z - RESIDENT_HALF);
      this.hi.set(x + RESIDENT_HALF, y + 0.1, z + RESIDENT_HALF);
      if (!reaches(r.planes, r.planeCount, this.lo, this.hi, MARGIN)) continue;
      r.shows[i] = 1;
      r.showsAnyone = true;
    }
  }

  /** How many reflections may be live at once, the render targets of those reflecting now, mirrors in the world and those reflecting now (benchmark reports). */
  get capacity(): number {
    return this.pool.length;
  }

  /**
   * Draws the reflections once, so the clip-plane shader variants they need (and their render
   * targets) are made before the game is shown. With `size`, the draw is at that size (the first
   * full-size draw then only fills pixels). Returns the undo.
   */
  warm(size = 0): () => void {
    const forced: Item[] = [];
    let textures = this.reflecting;
    // (The game's camera may not have a mirror in view yet: rank the first ones anyway.)
    if (!textures.length) {
      for (let i = 0; i < Math.min(this.items.length, this.pool.length); i++) {
        const item = this.items[i];
        if (item.slot >= 0) continue;
        item.slot = i;
        this.reflection(i);
        forced.push(item);
      }
      textures = this.reflecting;
    }
    const saved: { texture: MirrorTexture; size: { width: number; height: number } }[] = [];
    for (const texture of textures) {
      const now = texture.getSize();
      saved.push({ texture, size: { width: now.width, height: now.height } });
      if (size > 0) texture.resize({ width: size, height: size });
      texture.resetRefreshCounter();
      texture.render();
    }
    return () => {
      if (size > 0) for (const { texture, size: old } of saved) texture.resize(old);
      for (const item of forced) item.slot = -1;
    };
  }

  get reflecting(): MirrorTexture[] {
    return this.items.filter((item) => item.slot >= 0).map((item) => this.pool[item.slot]?.texture).filter((t): t is MirrorTexture => !!t);
  }

  /** The reflections' render targets made now. */
  get textures(): MirrorTexture[] {
    return this.pool.filter((r): r is Reflection => !!r).map((r) => r.texture);
  }

  get count(): number {
    return this.instances.length;
  }

  get live(): number {
    return this.items.filter((item) => item.slot >= 0).length;
  }

  /** Where the mirrors are and which way they face (the benchmark's mirror camera). */
  get spots(): { x: number; y: number; z: number; nx: number; ny: number; nz: number }[] {
    const face = new Matrix();
    const c = new Vector3();
    const n = new Vector3();
    return this.instances.map(({ matrices, index, surfaces }) => {
      Matrix.FromArrayToRef(matrices, index * 16, face);
      surfaces[0].matrix.multiplyToRef(face, face);
      Vector3.TransformCoordinatesFromFloatsToRef(0, 0, 0, face, c);
      Vector3.TransformNormalFromFloatsToRef(0, 0, 1, face, n).normalize();
      return { x: c.x, y: c.y, z: c.z, nx: n.x, ny: n.y, nz: n.z };
    });
  }

  dispose(): void {
    this.scene.getEngine().onResizeObservable.removeCallback(this.resize);
    this.set([]);
    this.release(Infinity);
    this.square.dispose();
    this.disc.dispose();
  }
}

/** The unit square's corners, in order round it. */
const SQUARE = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]] as const;

/** Whether the box `lo`–`hi` reaches into every one of the first `count` planes (inside positive), with `margin` to spare. */
function reaches(planes: readonly Plane[], count: number, lo: Vector3, hi: Vector3, margin: number): boolean {
  for (let p = 0; p < count; p++) {
    const { normal: n, d } = planes[p];
    const far = n.x * (n.x >= 0 ? hi.x : lo.x) + n.y * (n.y >= 0 ? hi.y : lo.y) + n.z * (n.z >= 0 ? hi.z : lo.z) + d;
    if (far < -margin) return false;
  }
  return true;
}

/** Whether two matrices are the same to within `STILL`. */
function same(a: Matrix, b: Matrix): boolean {
  const x = a.m;
  const y = b.m;
  for (let i = 0; i < 16; i++) if (Math.abs(x[i] - y[i]) > STILL * Math.max(1, Math.abs(x[i]))) return false;
  return true;
}

/** The unit square, or the unit-diameter disc, in the xy plane facing +z (never drawn itself; its clones are). */
function unitMesh(scene: Scene, name: string, round: boolean): Mesh {
  const mesh = new Mesh(name, scene);
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
  const normals = positions.map((_, i) => (i % 3 === 2 ? 1 : 0));
  const data = new VertexData();
  data.positions = positions;
  data.indices = indices;
  data.normals = normals;
  data.applyToMesh(mesh);
  mesh.setEnabled(false);
  return mesh;
}
