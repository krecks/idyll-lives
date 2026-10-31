#!/usr/bin/env bun
// Records the Babylon.js 9.30 fixtures the tests compare the game's own code against
// (`tests/fixtures/babylon/*.json`, read through `babylon.ts`): Babylon's matrix and quaternion maths
// (math, birds, residents), its colour curves and vignette uniforms (imageProcessing), its mesh builders
// and the Babylon renderer's primitive parts and mirror faces (modelParts), and its CPU particle system
// set up as the Babylon renderer set it up (buildFx), each for the tests' inputs, which are recorded
// with them.
//
// The game no longer has Babylon (Stage 11 of the PlayCanvas port removed it), so this needs a Babylon
// reference checkout: the last commit with the Babylon renderer, f15b839b (`render/babylon/` and
// `@babylonjs/core` 9.30.0 installed), e.g.
//   git worktree add bench-results/babylon-f15b839b f15b839b && (cd bench-results/babylon-f15b839b/web && bun install)
//   cd web && bun tests/fixtures/record-babylon.ts --ref ../bench-results/babylon-f15b839b/web
// Babylon and its renderer come from the checkout; the inputs and the shared code they go through
// (the build effects' bursts, `ParticleSettings`) from this tree. (An earlier commit won't do: the
// renderer gained `particleSystem`, `partMesh` and `mirrorSurface` during the port.)
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { f32, f64, u32, type Numbers } from './babylon';

const args = process.argv.slice(2);
const refAt = args.indexOf('--ref');
if (refAt < 0) throw new Error('usage: bun tests/fixtures/record-babylon.ts --ref <web/ of a checkout with Babylon (f15b839b)>');
const REF = resolve(args[refAt + 1]);
const OUT = new URL('./babylon/', import.meta.url).pathname;
mkdirSync(OUT, { recursive: true });

/* eslint-disable @typescript-eslint/no-explicit-any -- Babylon is imported by path, untyped. */
/** A module of the checkout's `@babylonjs/core`. */
const bjs = (path: string): Promise<any> => import(`${REF}/node_modules/@babylonjs/core/${path}.js`);
const { Matrix, Quaternion, Vector3 } = await bjs('Maths/math.vector');
const { Color3, Color4 } = await bjs('Maths/math.color');

/** A fixed sequence of numbers in 0..1 (the same every run). */
function numbers(seed: number): () => number {
  let s = seed >>> 0;
  return () => (s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32;
}
const arr = (m: any): number[] => Array.from(m.asArray() as Float32Array);
const quat = (q: any): number[] => [q.x, q.y, q.z, q.w];

function write(name: string, data: unknown): void {
  const text = JSON.stringify(data);
  writeFileSync(`${OUT}${name}.json`, `${text}\n`);
  console.log(`${name}.json (${(text.length / 1024).toFixed(0)} KB)`);
}

// --- math: render/shared/math.ts ---
{
  const u = numbers(7);
  const r = () => u() * 8 - 4;
  const CASES = 50;
  const many = <T>(fn: () => T) => Array.from({ length: CASES }, fn);
  const composed = (s: number[], q: number[], t: number[]) => Matrix.Compose(new Vector3(...s), new Quaternion(...q), new Vector3(...t));
  const anyMatrix = () => composed([r(), r(), r()], quat(Quaternion.RotationYawPitchRoll(r(), r(), r())), [r(), r(), r()]);
  const translation = [r(), r(), r()];
  write('math', {
    rotations: many(() => {
      const a = r();
      return { a, x: arr(Matrix.RotationX(a)), y: arr(Matrix.RotationY(a)), z: arr(Matrix.RotationZ(a)) };
    }),
    translation: { at: translation, translation: arr(Matrix.Translation(...translation)), scaling: arr(Matrix.Scaling(...translation)) },
    quaternions: many(() => {
      const [yaw, pitch, roll, ax, ay, az, angle] = [r(), r(), r(), r(), r(), r(), r()];
      return {
        yaw,
        pitch,
        roll,
        yawPitchRoll: quat(Quaternion.RotationYawPitchRoll(yaw, pitch, roll)),
        // `Quaternion.FromEulerAngles(pitch, yaw, roll)`.
        euler: quat(Quaternion.FromEulerAngles(pitch, yaw, roll)),
        axis: [ax, ay, az],
        angle,
        axisAngle: quat(Quaternion.RotationAxis(new Vector3(ax, ay, az), angle)),
      };
    }),
    compose: many(() => {
      const [s, q, t] = [[r(), r(), r()], quat(Quaternion.RotationYawPitchRoll(r(), r(), r())), [r(), r(), r()]];
      return { s, q, t, m: arr(composed(s, q, t)) };
    }),
    multiply: many(() => {
      const [a, b] = [anyMatrix(), anyMatrix()];
      return { a: arr(a), b: arr(b), ab: arr(a.multiply(b)) };
    }),
    invert: many(() => {
      const a = composed([1 + Math.abs(r()), 1 + Math.abs(r()), 1 + Math.abs(r())], quat(Quaternion.RotationYawPitchRoll(r(), r(), r())), [r(), r(), r()]);
      return { a: arr(a), inverse: arr(Matrix.Invert(a)) };
    }),
    transform: many(() => {
      const a = anyMatrix();
      const p = [r(), r(), r()];
      const point = Vector3.TransformCoordinatesFromFloatsToRef(p[0], p[1], p[2], a, new Vector3());
      const normal = Vector3.TransformNormalFromFloatsToRef(p[0], p[1], p[2], a, new Vector3());
      return { m: arr(a), p, point: [point.x, point.y, point.z], normal: [normal.x, normal.y, normal.z] };
    }),
    // sRGB to linear, `Color3.toLinearSpace`, over the whole range.
    toLinear: Array.from({ length: 101 }, (_, i) => [i / 100, new Color3(i / 100, i / 100, i / 100).toLinearSpace().r]),
  });
}

// --- imageProcessing: render/shared/imageProcessing.ts ---
{
  const { ColorCurves } = await bjs('Materials/colorCurves');
  const { ImageProcessingConfiguration } = await bjs('Materials/imageProcessingConfiguration');
  const { LOOK } = await import('../../src/render/look');
  const { NEUTRAL_CURVES } = await import('../../src/render/shared/imageProcessing');
  /** What Babylon hands an effect: `setFloat4`/`setFloat2`/`setFloat` calls, by uniform. */
  const recorder = () => {
    const values = new Map<string, number[]>();
    const effect = {
      setFloat4: (name: string, ...v: number[]) => values.set(name, v),
      setFloat2: (name: string, ...v: number[]) => values.set(name, v),
      setFloat: (name: string, v: number) => values.set(name, [v]),
      setMatrix3x3: () => {},
      getEngine: () => ({ getRenderWidth: () => 1920, getRenderHeight: () => 1080 }),
    };
    return { values, effect };
  };
  const curves = (settings: Record<string, number>) => {
    const c = new ColorCurves();
    Object.assign(c, settings);
    const { values, effect } = recorder();
    // `_bind` is the method `ImageProcessingConfiguration.bind` calls (`ColorCurvesBind`).
    c._bind(effect);
    return { settings, neutral: values.get('vCameraColorCurveNeutral'), positive: values.get('vCameraColorCurvePositive'), negative: values.get('vCameraColorCurveNegative') };
  };
  const r = numbers(11);
  const random = Array.from({ length: 60 }, () => {
    const c: Record<string, number> = { ...NEUTRAL_CURVES };
    for (const key of Object.keys(c)) c[key] = key.endsWith('Hue') ? r() * 400 - 20 : r() * 240 - 120;
    return curves(c);
  });
  const vignette = [];
  for (const [weight, stretch, color] of [
    [LOOK.vignette, 0.5, [0.02, 0.02, 0.04]],
    [0, 0.5, [0.02, 0.02, 0.04]],
    [1.5, 0, [0.3, 0.1, 0]],
    [4, 1, [0, 0, 0]],
  ] as [number, number, number[]][]) {
    for (const [w, h] of [[1920, 1080], [1280, 1024], [800, 1200], [1, 1]]) {
      const config = new ImageProcessingConfiguration();
      config.vignetteEnabled = true;
      config.vignetteStretch = stretch;
      config.vignetteWeight = weight;
      config.vignetteColor = new Color4(color[0], color[1], color[2], 0);
      config.outputTextureWidth = w;
      config.outputTextureHeight = h;
      const { values, effect } = recorder();
      // (The post-process hands its own aspect, its texture's: the same picture size.)
      config.bind(effect, w / h);
      vignette.push({ w, h, weight, stretch, color, settings1: values.get('vignetteSettings1'), settings2: values.get('vignetteSettings2') });
    }
  }
  const look = { ...NEUTRAL_CURVES, globalSaturation: LOOK.saturation, highlightsHue: LOOK.highlightsHue, highlightsDensity: LOOK.highlightsDensity, shadowsHue: LOOK.shadowsHue, shadowsDensity: LOOK.shadowsDensity };
  write('imageProcessing', { look: curves(look), neutral: curves({ ...NEUTRAL_CURVES }), curves: random, vignette });
}

// --- modelParts: render/shared/modelParts.ts ---
{
  // (The Babylon renderer's models module sets up its loaders against the page's address.)
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '' }, $state: <T>(v: T) => v, $effect: Object.assign(() => {}, { root: () => () => {} }) });
  const { NullEngine } = await bjs('Engines/nullEngine');
  const { Scene } = await bjs('scene');
  const { VertexBuffer } = await bjs('Buffers/buffer');
  const { VertexData } = await bjs('Meshes/mesh.vertexData');
  const { CreateCylinderVertexData } = await bjs('Meshes/Builders/cylinderBuilder');
  const { CreateSphereVertexData } = await bjs('Meshes/Builders/sphereBuilder');
  const { CreateTorusVertexData } = await bjs('Meshes/Builders/torusBuilder');
  const { CreateCapsuleVertexData } = await bjs('Meshes/Builders/capsuleBuilder');
  const { CreateIcoSphereVertexData } = await bjs('Meshes/Builders/icoSphereBuilder');
  const { mirrorSurface, partMesh } = await import(`${REF}/src/render/babylon/models.ts`);
  // (As Float32Arrays: the tests compare vertices to 1e-5, and the meshes upload them as such.)
  const vertices = (a: ArrayLike<number> | null | undefined): Numbers | null => (a ? f32(a) : null);
  const shape = (args: unknown, d: any) => ({ args, positions: vertices(d.positions), normals: vertices(d.normals), indices: u32(d.indices ?? []) });
  const cylinders = [[0.8, 1, 1, 20], [0.5, 0, 1, 16], [1.2, 0.4, 1, 8], [0.3, 0.6, 0.2, 5], [2, 1, 0, 32]].map(([h, top, bottom, n]) => shape([h, top, bottom, n], CreateCylinderVertexData({ height: h, diameterTop: top, diameterBottom: bottom, tessellation: n })));
  const spheres = [[0.3, 0.3, 0.3, 12], [0.5, 0.2, 0.8, 6], [1, 1, 1, 16], [0.1, 0.4, 0.2, 3]].map(([x, y, z, n]) => shape([x, y, z, n], CreateSphereVertexData({ diameterX: x, diameterY: y, diameterZ: z, segments: n })));
  const tori = [[0.3, 0.05, 24], [0.6, 0.1, 12], [1, 0.3, 6]].map(([d, t, n]) => shape([d, t, n], CreateTorusVertexData({ diameter: d, thickness: t, tessellation: n })));
  const capsules = [[0.9, 0.12, 16, 6], [0.5, 0.2, 8, 2], [1.6, 0.05, 12, 4]].map(([h, r, n, caps]) => shape([h, r, n, caps], CreateCapsuleVertexData({ height: h, radius: r, tessellation: n, capSubdivisions: caps })));
  const icospheres = [[0.5, 1], [0.5, 2], [0.5, 3], [1.2, 2]].map(([r, n]) => shape([r, n], CreateIcoSphereVertexData({ radius: r, subdivisions: n, flat: false })));
  // Normals of an icosphere's triangles, smoothed (`VertexData.ComputeNormals`).
  const ico = CreateIcoSphereVertexData({ radius: 0.5, subdivisions: 2, flat: false });
  const normals: number[] = [];
  VertexData.ComputeNormals(ico.positions, ico.indices, normals);
  const computeNormals = { positions: f32(ico.positions), indices: u32(ico.indices), normals: f32(normals) };

  const scene = new Scene(new NullEngine());
  const PARTS = [
    { shape: 'box', size: [0.8, 0.4, 0.5], at: [0, 0.2, 0], color: '#B8B2AA' },
    { shape: 'box', size: [0.1, 1.2, 0.04], at: [0.3, 0.6, -0.2], color: '#3A6EA5', rotate: [10, 35, -5], radius: 0.02 },
    { shape: 'box', size: [1.4, 0.06, 0.7], at: [0, 0.74, 0], color: '#6B4430', rotate: [0, -20, 0], radius: 0.03 },
    { shape: 'cylinder', size: [0.3, 0.7, 0.2], at: [0.1, 0.35, 0], color: '#C08A64', taper: 0.7 },
    { shape: 'cylinder', size: [0.05, 0.9, 0.05], at: [-0.3, 0.45, 0.2], color: '#1A1A1A' },
    { shape: 'cone', size: [0.5, 0.6, 0.5], at: [0, 1, 0], color: '#2F3236', rotate: [90, 0, 0] },
    { shape: 'sphere', size: [0.3, 0.2, 0.4], at: [-0.2, 0.5, 0.1], color: '#FFFFFF', segments: 8 },
    { shape: 'sphere', size: [0.6, 0.6, 0.6], at: [0, 0.3, 0], color: '#E8C0A0' },
    { shape: 'capsule', size: [0.2, 0.8, 0.3], at: [0, 0.4, 0], color: '#123456', rotate: [0, 0, 90] },
    { shape: 'blob', size: [0.9, 0.7, 0.8], at: [0, 1.4, 0], color: '#4F7A2E' },
    { shape: 'torus', size: [0.4, 0.05, 0.3], at: [0, 0.9, 0], color: '#AA8800' },
    { shape: 'lathe', size: [0.3, 0.4, 0.3], at: [0, 0.2, 0], color: '#DDDDDD', profile: [[0, -0.5], [0.4, -0.5], [0.5, 0.2], [0.3, 0.5]] },
  ];
  const parts = PARTS.flatMap((part, index) =>
    ([0, 1] as const).map((lod) => {
      const mesh = partMesh(scene, part, lod, index, 0.25);
      const out = {
        part,
        lod,
        index,
        alpha: 0.25,
        positions: vertices(mesh.getVerticesData(VertexBuffer.PositionKind)),
        normals: vertices(mesh.getVerticesData(VertexBuffer.NormalKind)),
        uvs: vertices(mesh.getVerticesData(VertexBuffer.UVKind)),
        colors: vertices(mesh.getVerticesData(VertexBuffer.ColorKind)),
        indices: u32(mesh.getIndices() ?? []),
      };
      mesh.dispose();
      return out;
    }),
  );
  const mirrors = [
    { shape: 'box', size: [0.6, 0.9, 0.02], at: [0, 1.2, -0.2], color: '#FFFFFF', mirror: 'front', rotate: [0, 15, 0] },
    { shape: 'cylinder', size: [0.5, 0.02, 0.5], at: [0, 1.4, 0], color: '#FFFFFF', mirror: 'back', rotate: [90, 0, 0] },
    { shape: 'box', size: [0.02, 0.5, 0.4], at: [0.3, 1, 0], color: '#FFFFFF' },
    { shape: 'box', size: [0.8, 1.6, 0.03], at: [1, 0.9, 0.4], color: '#FFFFFF', mirror: 'back', rotate: [5, -70, 2] },
    { shape: 'cylinder', size: [0.4, 0.4, 0.02], at: [0, 1.5, 0], color: '#FFFFFF', mirror: 'front' },
  ].map((part) => {
    const m = mirrorSurface(part);
    return { part, round: m.round, matrix: f32(m.matrix.asArray()) };
  });
  scene.dispose();
  write('modelParts', { cylinders, spheres, tori, capsules, icospheres, computeNormals, parts, mirrors });
}

// --- buildFx: render/shared/buildFx.ts ParticleSim against Babylon's ParticleSystem ---
{
  const { NullEngine } = await bjs('Engines/nullEngine');
  const { Scene } = await bjs('scene');
  const { ParticleSystem } = await bjs('Particles/particleSystem');
  const { particleSystem } = await import(`${REF}/src/render/babylon/buildFx.ts`);
  const { animationRatio, Bursts, DUST, SPARKLES } = await import('../../src/render/shared/buildFx');
  const FX = { kind: 'place', x: 3, z: 5, w: 2, d: 1, y: 2.8 } as const;
  /** Frames (ms), and a burst before some of them: [frame, count, y0, y1]. */
  const FRAMES = [16.7, 16.7, 33.4, 8.3, 50, 16.7, 16.7, 100, 16.7, 16.7, 16.7, 25, 16.7, 16.7, 16.7, 16.7, 400, 16.7, 16.7, 400, 400, 400];
  const BURSTS = [
    [0, 40, 0.15, 0.7],
    [4, 22, 0.05, 0.4],
    [9, 300, 0.3, 0.8],
  ];
  const SEED = 7;
  const runs = [SPARKLES, DUST].map((settings) => {
    const random = Math.random;
    Math.random = numbers(SEED);
    try {
      const engine = new NullEngine();
      const scene = new Scene(engine);
      const ps = particleSystem(scene, ParticleSystem, settings, null);
      const bursts = new Bursts();
      ps.emitter = Vector3.Zero();
      ps.startPositionFunction = (_world: unknown, position: { x: number; y: number; z: number }) => bursts.start(position);
      ps.start();
      const frames: Numbers[] = [];
      FRAMES.forEach((ms, f) => {
        for (const [at, count, y0, y1] of BURSTS) {
          if (at !== f) continue;
          bursts.add(FX, count, y0, y1);
          ps.manualEmitCount = Math.max(0, ps.manualEmitCount) + count;
        }
        // (`animate(true)` steps as a frame does, by `preWarmStepOffset` in place of the scene's animation ratio.)
        ps.preWarmStepOffset = animationRatio(ms);
        ps.animate(true);
        frames.push(f32(ps.particles.flatMap((p: any) => [p.position.x, p.position.y, p.position.z, p.color.r, p.color.g, p.color.b, p.color.a, p.size * p.scale.x, p.angle])));
      });
      ps.dispose();
      scene.dispose();
      engine.dispose();
      return { settings, frames };
    } finally {
      Math.random = random;
    }
  });
  write('buildFx', { seed: SEED, fx: FX, frames: FRAMES, bursts: BURSTS, runs });
}

// --- birds: a bird's parts' matrices (render/shared/birds.ts `write`) ---
{
  const r = numbers(11);
  const s = (k = 1) => (r() * 2 - 1) * k;
  const FIELDS = ['x', 'y', 'z', 'yaw', 'pitch', 'roll', 'hy', 'hp', 'hr', 'bob', 'tail', 'puff', 'spread', 'flap', 'size', 'centre', ...[1, 2, 3, 4].flatMap((k) => [`pivot${k}x`, `pivot${k}y`, `pivot${k}z`])];
  const CASES = 300;
  const inputs: number[] = [];
  const parts: number[] = [];
  const [qTurn, mWorld, mBase, mPose, mTurn, mOut] = [new Quaternion(), new Matrix(), new Matrix(), new Matrix(), new Matrix(), new Matrix()];
  const about = (m: any, p: any, dz = 0) => {
    const v = Vector3.TransformNormal(p, m);
    return m.setTranslationFromFloats(p.x - v.x, p.y - v.y, p.z - v.z + dz);
  };
  for (let n = 0; n < CASES; n++) {
    const v: Record<string, number> = { x: s(30) + 30, y: r() * 8, z: s(30) + 30, yaw: s(7), pitch: s(1), roll: s(0.7), hy: s(2.5), hp: s(1), hr: s(0.5), bob: s(0.02), tail: s(0.6), puff: 1 + s(0.1), spread: n % 3 ? 1 : 0, flap: s(1.2), size: 0.6 + r() * 0.8, centre: 0.02 + r() * 0.12 };
    for (const k of [1, 2, 3, 4]) [v[`pivot${k}x`], v[`pivot${k}y`], v[`pivot${k}z`]] = [s(0.1), 0.05 + s(0.05), s(0.1)];
    inputs.push(...FIELDS.map((f) => v[f]));
    // As the Babylon renderer's birds wrote them.
    Quaternion.RotationYawPitchRollToRef(v.yaw, v.pitch, v.roll, qTurn);
    Matrix.ComposeToRef(new Vector3(v.size, v.size, v.size), qTurn, new Vector3(v.x, v.y + v.centre * v.size, v.z), mWorld);
    Matrix.TranslationToRef(0, -v.centre, 0, mPose).multiplyToRef(mWorld, mBase);
    const want = (pose: any) => parts.push(...(pose.multiplyToRef(mBase, mOut).m as Float32Array));
    const pivot = (k: number) => new Vector3(v[`pivot${k}x`], v[`pivot${k}y`], v[`pivot${k}z`]);
    want(about(Matrix.ScalingToRef(v.puff, v.puff, v.puff, mPose), new Vector3(0, v.centre, 0)));
    want(about(Matrix.RotationYawPitchRollToRef(v.hy, v.hp, v.hr, mPose), pivot(1), v.bob));
    want(about(Matrix.RotationXToRef(v.tail, mPose), pivot(2)));
    for (let part = 3; part <= 4; part++) want(about(Matrix.ScalingToRef(v.spread, v.spread, v.spread, mPose).multiplyToRef(Matrix.RotationZToRef((part === 3 ? -1 : 1) * v.flap, mTurn), mPose), pivot(part)));
  }
  write('birds', { fields: FIELDS, cases: CASES, inputs: f64(inputs), parts: f32(parts) });
}

// --- residents: the matrix maths in render/shared/characters/rig.ts ---
{
  const r = numbers(5);
  const s = (k = 1) => (r() * 2 - 1) * k;
  const CASES = 200;
  const [q, t, ride, composed, multiplied, rotation] = [[], [], [], [], [], []] as number[][];
  for (let n = 0; n < CASES; n++) {
    const qn = Quaternion.RotationYawPitchRoll(s(3), s(1), s(0.5));
    const tn = new Vector3(s(9), s(2), s(9));
    const rn = Matrix.Compose(new Vector3(1, 1, 1), Quaternion.RotationYawPitchRoll(s(0.3), s(0.3), s(0.3)), new Vector3(s(), s(), s()));
    const theirs = Matrix.Compose(new Vector3(1, 1, 1), qn, tn);
    q.push(...quat(qn));
    t.push(tn.x, tn.y, tn.z);
    ride.push(...(rn.m as Float32Array));
    composed.push(...(theirs.m as Float32Array));
    theirs.multiplyToRef(rn, theirs);
    multiplied.push(...(theirs.m as Float32Array));
    const dq = new Quaternion();
    theirs.decompose(new Vector3(), dq, new Vector3());
    rotation.push(...quat(dq));
  }
  // A bone's height at rest: the inverse bind matrix inverted.
  const b = numbers(9);
  const [inverseBinds, heights] = [[], []] as number[][];
  const m = new Float32Array(32);
  for (let n = 0; n < CASES; n++) {
    const a = Matrix.Compose(new Vector3(1, 1, 1), Quaternion.RotationYawPitchRoll(b() * 6, b() - 0.5, b() - 0.5), new Vector3(b() - 0.5, b() * 1.8, b() - 0.5)).invert();
    m.set(a.m, 16);
    inverseBinds.push(...(a.m as Float32Array));
    heights.push(Matrix.FromArray(m, 16).invert().m[13]);
  }
  write('residents', {
    cases: CASES,
    compose: { q: f64(q), t: f64(t), ride: f32(ride), composed: f32(composed), multiplied: f32(multiplied), rotation: f64(rotation) },
    bindY: { inverseBinds: f32(inverseBinds), heights: f64(heights) },
  });
}
