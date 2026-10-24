/**
 * Buy-mode feedback in the world on Babylon (what plays when, the rings' spread and the pops'
 * spring are `shared/buildFx.ts`'s): a ring that spreads over the footprint, sparkles for new and
 * upgraded furniture, dust for walls going up or coming down and a puff for things sold; and
 * the "pop" that springs newly placed furniture into place (its instance matrix is scaled for a
 * moment, see `PopTarget`). Everything is pooled and costs nothing when idle.
 */

import { Color3, Color4, DynamicTexture, Matrix, MeshBuilder, StandardMaterial, Vector3, type Mesh, type ParticleSystem, type Scene } from './core';
import { BuildEffectsCore, Bursts, DUST, RING_COLORS, SPARKLES, type ParticleKind, type ParticleSettings, type RingState } from '../shared/buildFx';
import type { BuildEffect } from '../types';

/** Babylon's particles are only needed once something is built: loaded with the first effect. */
type Particles = typeof import('@babylonjs/core/Particles/particleSystem');
let particles: Promise<Particles> | null = null;
const loadParticles = () => (particles ??= import('@babylonjs/core/Particles/particleSystem'));

/** One placed object's instance: where its matrix lives in the template's buffer. */
export interface PopTarget {
  meshes: Mesh[];
  matrices: Float32Array;
  index: number;
  /** Its moving parts (a fridge's doors), which follow the matrix (`writeMoving`). */
  moving?: MovingTarget;
}

/** An instance's moving parts (`ModelMotion`): each part's meshes draw from a buffer of their own. */
export interface MovingTarget {
  /** Per part: its meshes and their buffer (the same index as the instance's). */
  parts: { meshes: Mesh[]; matrices: Float32Array }[];
  /** Per part: its pose now (16 floats each, identity at rest): where it is, before the instance's matrix. */
  poses: Float32Array;
}

/**
 * Writes an instance's moving parts after its matrix: each part's pose, then the instance's
 * placement. The buffers go up now, or, given `updated`, their meshes go in it for the caller to
 * send up once (many instances of one model moving in a frame).
 */
export function writeMoving(target: PopTarget, updated?: Set<Mesh>): void {
  const moving = target.moving;
  if (!moving) return;
  const o = target.index * 16;
  Matrix.FromArrayToRef(target.matrices, o, mBase);
  moving.parts.forEach(({ meshes, matrices }, k) => {
    Matrix.FromArrayToRef(moving.poses, k * 16, mPose);
    mPose.multiplyToRef(mBase, mPose);
    mPose.copyToArray(matrices, o);
    for (const mesh of meshes) {
      if (updated) {
        updated.add(mesh);
        continue;
      }
      if (mesh.isDisposed()) continue;
      mesh.thinInstanceAllowAutomaticStaticBufferRecreation = true;
      mesh.thinInstanceBufferUpdated('matrix');
    }
  });
}
const mBase = new Matrix();
const mPose = new Matrix();

/** The effects (`shared/buildFx.ts`) on Babylon: a ground mesh per ring, a Babylon particle system each for the sparkles and the dust. */
export class BuildEffects extends BuildEffectsCore<PopTarget> {
  private readonly ringMeshes: Mesh[] = [];
  private readonly ringMaterials = new Map<string, StandardMaterial>();
  private readonly ringTexture: DynamicTexture;
  private sparkles: ParticleSystem | null = null;
  private dust: ParticleSystem | null = null;
  private readonly bursts = new Map<ParticleSystem, Bursts>();
  private disposed = false;

  constructor(private readonly scene: Scene) {
    super();
    this.ringTexture = ringTexture(scene);
  }

  dispose(): void {
    this.disposed = true;
    for (const mesh of this.ringMeshes) mesh.dispose();
    this.sparkles?.dispose();
    this.dust?.dispose();
  }

  protected placeRing(slot: number, ring: RingState): void {
    let mesh = this.ringMeshes[slot];
    if (!mesh) {
      mesh = this.ringMeshes[slot] = MeshBuilder.CreateGround('buildRing', { width: 1, height: 1 }, this.scene);
      mesh.isPickable = false;
      mesh.alwaysSelectAsActiveMesh = true;
    }
    mesh.material = this.ringMaterial(RING_COLORS[ring.kind]);
    mesh.position.set(ring.x, ring.y, ring.z);
    mesh.setEnabled(true);
  }

  protected drawRing(slot: number, ring: RingState): void {
    const mesh = this.ringMeshes[slot];
    mesh.scaling.set(ring.scale, 1, ring.scale);
    mesh.visibility = ring.alpha;
    if (!ring.on) mesh.setEnabled(false);
  }

  protected popStarted(target: PopTarget): void {
    for (const mesh of target.meshes) mesh.thinInstanceAllowAutomaticStaticBufferRecreation = true;
  }

  protected popWritten(target: PopTarget): void {
    for (const mesh of target.meshes) {
      if (mesh.isDisposed()) continue;
      mesh.thinInstanceBufferUpdated('matrix');
    }
    writeMoving(target);
  }

  private ringMaterial(color: string): StandardMaterial {
    let mat = this.ringMaterials.get(color);
    if (!mat) {
      mat = new StandardMaterial(`buildRing${color}`, this.scene);
      mat.disableLighting = true;
      mat.diffuseColor = Color3.Black();
      mat.specularColor = Color3.Black();
      mat.emissiveColor = Color3.FromHexString(color);
      mat.opacityTexture = this.ringTexture;
      mat.disableDepthWrite = true;
      mat.zOffset = -6;
      this.ringMaterials.set(color, mat);
    }
    return mat;
  }

  /** A burst on the sparkle or dust system; the very first one waits for the particle module. */
  protected emit(kind: ParticleKind, fx: BuildEffect, count: number, y0: number, y1: number): void {
    const ready = kind === 'sparkle' ? this.sparkles : this.dust;
    if (ready) return this.burst(ready, fx, count, y0, y1);
    void loadParticles().then(({ ParticleSystem }) => {
      if (this.disposed) return;
      const ps = kind === 'sparkle' ? (this.sparkles ??= this.system(ParticleSystem, kind)) : (this.dust ??= this.system(ParticleSystem, kind));
      this.burst(ps, fx, count, y0, y1);
    });
  }

  /** Queues `count` particles inside the effect's footprint, between heights `y0` and `y1`. */
  private burst(ps: ParticleSystem, fx: BuildEffect, count: number, y0: number, y1: number): void {
    this.bursts.get(ps)!.add(fx, count, y0, y1);
    ps.manualEmitCount = Math.max(0, ps.manualEmitCount) + count;
    if (!ps.isStarted()) ps.start();
  }

  /** A particle system whose particles start in the queued bursts' boxes (in order). */
  private system(PS: Particles['ParticleSystem'], kind: ParticleKind): ParticleSystem {
    const ps = kind === 'sparkle' ? particleSystem(this.scene, PS, SPARKLES, sparkleTexture(this.scene)) : particleSystem(this.scene, PS, DUST, puffTexture(this.scene));
    const bursts = new Bursts();
    this.bursts.set(ps, bursts);
    ps.emitter = Vector3.Zero();
    ps.startPositionFunction = (_world, position) => bursts.start(position);
    return ps;
  }
}

function ringTexture(scene: Scene): DynamicTexture {
  const n = 256;
  const tex = new DynamicTexture('buildRing', n, scene, true);
  const ctx = tex.getContext() as CanvasRenderingContext2D;
  const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
  g.addColorStop(0, 'rgba(255,255,255,0)');
  g.addColorStop(0.5, 'rgba(255,255,255,0.12)');
  g.addColorStop(0.76, 'rgba(255,255,255,1)');
  g.addColorStop(0.9, 'rgba(255,255,255,0.85)');
  g.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, n, n);
  tex.update();
  tex.hasAlpha = true;
  return tex;
}

/** A soft four-pointed twinkle. */
function sparkleTexture(scene: Scene): DynamicTexture {
  const n = 64;
  const tex = new DynamicTexture('buildSparkle', n, scene, true);
  const ctx = tex.getContext() as CanvasRenderingContext2D;
  ctx.clearRect(0, 0, n, n);
  const glow = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
  glow.addColorStop(0, 'rgba(255,255,255,1)');
  glow.addColorStop(0.25, 'rgba(255,255,255,0.5)');
  glow.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = glow;
  ctx.fillRect(0, 0, n, n);
  ctx.fillStyle = 'rgba(255,255,255,0.95)';
  ctx.beginPath();
  const c = n / 2;
  ctx.moveTo(c, 2);
  ctx.quadraticCurveTo(c + 3, c - 3, n - 2, c);
  ctx.quadraticCurveTo(c + 3, c + 3, c, n - 2);
  ctx.quadraticCurveTo(c - 3, c + 3, 2, c);
  ctx.quadraticCurveTo(c - 3, c - 3, c, 2);
  ctx.fill();
  tex.update();
  tex.hasAlpha = true;
  return tex;
}

function puffTexture(scene: Scene): DynamicTexture {
  const n = 64;
  const tex = new DynamicTexture('buildPuff', n, scene, true);
  const ctx = tex.getContext() as CanvasRenderingContext2D;
  ctx.clearRect(0, 0, n, n);
  const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
  g.addColorStop(0, 'rgba(255,255,255,0.9)');
  g.addColorStop(0.5, 'rgba(255,255,255,0.45)');
  g.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, n, n);
  tex.update();
  tex.hasAlpha = true;
  return tex;
}

/** A Babylon particle system set up as `settings` say (`shared/buildFx.ts`; exported for `tests/buildFx.test.ts`). */
export function particleSystem(scene: Scene, PS: Particles['ParticleSystem'], settings: ParticleSettings, texture: DynamicTexture | null): ParticleSystem {
  const ps = new PS(settings.name, settings.capacity, scene);
  ps.particleTexture = texture;
  ps.blendMode = settings.blend === 'add' ? PS.BLENDMODE_ADD : PS.BLENDMODE_STANDARD;
  ps.color1 = new Color4(...settings.color1);
  ps.color2 = new Color4(...settings.color2);
  ps.colorDead = new Color4(...settings.colorDead);
  ps.minSize = settings.minSize;
  ps.maxSize = settings.maxSize;
  for (const [gradient, factor] of settings.sizeGradients ?? []) ps.addSizeGradient(gradient, factor);
  ps.minLifeTime = settings.minLifeTime;
  ps.maxLifeTime = settings.maxLifeTime;
  ps.emitRate = 0;
  ps.direction1 = new Vector3(...settings.direction1);
  ps.direction2 = new Vector3(...settings.direction2);
  ps.minEmitPower = settings.minEmitPower;
  ps.maxEmitPower = settings.maxEmitPower;
  ps.gravity = new Vector3(...settings.gravity);
  ps.minAngularSpeed = settings.minAngularSpeed;
  ps.maxAngularSpeed = settings.maxAngularSpeed;
  ps.updateSpeed = settings.updateSpeed;
  ps.isLocal = false;
  return ps;
}
