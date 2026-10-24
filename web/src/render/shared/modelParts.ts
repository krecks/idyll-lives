// Ported from Babylon.js 9.30: the mesh builders (`CreateCylinderVertexData`, `CreateSphereVertexData`,
// `CreateTorusVertexData`, `CreateCapsuleVertexData`, `CreateIcoSphereVertexData`) and `VertexData`'s
// `ComputeNormals`, `_FlipFaces` and `transform`. Changed for Idyll Lives: cut down to the options the
// parts use, without their UVs, on plain arrays.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The geometry of models, engine-neutral: primitive parts (`PrimitivePart`: boxes with rounded
 * edges, cylinders, cones, spheres, capsules, tori, lathes, lumpy blobs) as vertex arrays in model
 * space, and the operations a build does on a file's pieces (baking a transform in, merging,
 * resizing, fitting into a box, a mirror's face, the glowing faces). Game space (left-handed, y up)
 * and Babylon's conventions throughout, so either engine draws the same vertices: front faces wind
 * so that (a - b) x (c - b) points along the normal (`pushTri`), as Babylon's builders make them.
 *
 * The shapes are ports of Babylon.js's `CreateCylinderVertexData`, `CreateSphereVertexData`,
 * `CreateTorusVertexData`, `CreateCapsuleVertexData`, `CreateIcoSphereVertexData` and
 * `VertexData.ComputeNormals`, cut down to the options the parts use: the same vertices in the
 * same order (`web/tests/modelParts.test.ts`).
 * Their UVs are left out: every part gets metre-scale box-projected ones (`boxProjectUVs`).
 */

import type { PrimitivePart, Vec3 } from '../../assets/types';
import { compose, identity, invert, multiply, quatFromEuler, rotationX, rotationY, rotationZ, scaling, transformNormal, transformPoint, translation, type Mat4, type Quat } from './math';
import type { ExtraShapes, MaterialOverride, Resize } from './modelEntry';

/** Vertex values: a file's (32-bit), or a builder's (64-bit, as Babylon keeps a builder's arrays: a box-projected UV's axis then ties the same way). */
export type Floats = Float32Array | Float64Array;

/** A mesh's vertex arrays (Babylon's `VertexData`): positions, normals, UVs, RGBA colours, tangents (4 each), triangles. */
export interface VData {
  positions: Floats;
  normals: Floats | null;
  uvs?: Floats | null;
  colors?: Floats | null;
  tangents?: Floats | null;
  indices: Uint32Array;
}

/** The arrays of `data`, copied. */
export function copyData(data: VData): VData {
  return {
    positions: data.positions.slice(),
    normals: data.normals?.slice() ?? null,
    uvs: data.uvs?.slice() ?? null,
    colors: data.colors?.slice() ?? null,
    tangents: data.tangents?.slice() ?? null,
    indices: data.indices.slice(),
  };
}

/** `data` from plain arrays (a builder's): in 64-bit floats as Babylon keeps them, or 32-bit as it keeps an updatable mesh's (`f32`). */
function vdata(positions: ArrayLike<number>, normals: ArrayLike<number>, indices: ArrayLike<number>, f32 = false): VData {
  const F = f32 ? Float32Array : Float64Array;
  return { positions: F.from(positions), normals: F.from(normals), indices: Uint32Array.from(indices) };
}

/**
 * Babylon's `VertexData.transform`: positions through `m`, normals and tangents through its
 * rotation and scale (not normalised), and the triangles turned round when `m` mirrors.
 */
export function transformData(data: VData, m: Readonly<Mat4>): VData {
  const v = [0, 0, 0];
  const p = data.positions;
  for (let i = 0; i < p.length; i += 3) {
    transformPoint(m, p[i], p[i + 1], p[i + 2], v);
    p[i] = v[0];
    p[i + 1] = v[1];
    p[i + 2] = v[2];
  }
  for (const [values, stride] of [[data.normals, 3], [data.tangents, 4]] as const) {
    if (!values) continue;
    for (let i = 0; i < values.length; i += stride) {
      transformNormal(m, values[i], values[i + 1], values[i + 2], v);
      values[i] = v[0];
      values[i + 1] = v[1];
      values[i + 2] = v[2];
    }
  }
  if (determinant(m) < 0) flipFaces(data.indices);
  return data;
}

/** Babylon's `Mesh.bakeTransformIntoVertices`: as `transformData`, the normals (and tangents) normalised. */
export function bakeData(data: VData, m: Readonly<Mat4>): VData {
  const v = [0, 0, 0];
  const p = data.positions;
  for (let i = 0; i < p.length; i += 3) {
    transformPoint(m, p[i], p[i + 1], p[i + 2], v);
    p[i] = v[0];
    p[i + 1] = v[1];
    p[i + 2] = v[2];
  }
  for (const [values, stride] of [[data.normals, 3], [data.tangents, 4]] as const) {
    if (!values) continue;
    for (let i = 0; i < values.length; i += stride) {
      transformNormal(m, values[i], values[i + 1], values[i + 2], v);
      normalize3(v);
      values[i] = v[0];
      values[i + 1] = v[1];
      values[i + 2] = v[2];
    }
  }
  if (determinant(m) < 0) flipFaces(data.indices);
  return data;
}

/** Turns every triangle round (Babylon's `VertexData._FlipFaces`). */
export function flipFaces(indices: Uint32Array | Uint16Array | number[]): void {
  for (let i = 0; i + 2 < indices.length; i += 3) {
    const t = indices[i + 1];
    indices[i + 1] = indices[i + 2];
    indices[i + 2] = t;
  }
}

/** One mesh of several (`Mesh.MergeMeshes`): their vertices one after the other. An attribute some lack is filled with zeros. */
export function mergeData(list: readonly VData[]): VData {
  if (list.length === 1) return list[0];
  const count = list.reduce((n, d) => n + d.positions.length / 3, 0);
  const join = (key: 'normals' | 'uvs' | 'colors' | 'tangents', size: number): Float32Array | null => {
    if (!list.some((d) => d[key])) return null;
    const out = new Float32Array(count * size);
    let at = 0;
    for (const d of list) {
      const values = d[key];
      if (values) out.set(values, at);
      at += (d.positions.length / 3) * size;
    }
    return out;
  };
  const positions = new Float32Array(count * 3);
  const indices = new Uint32Array(list.reduce((n, d) => n + d.indices.length, 0));
  let base = 0;
  let at = 0;
  for (const d of list) {
    positions.set(d.positions, base * 3);
    for (let i = 0; i < d.indices.length; i++) indices[at + i] = d.indices[i] + base;
    at += d.indices.length;
    base += d.positions.length / 3;
  }
  return { positions, normals: join('normals', 3), uvs: join('uvs', 2), colors: join('colors', 4), tangents: join('tangents', 4), indices };
}

/** Babylon's `VertexData.ComputeNormals` (no options): each vertex's faces' normals, averaged. */
export function computeNormals(positions: ArrayLike<number>, indices: ArrayLike<number>, normals: number[] | Float32Array): void {
  for (let i = 0; i < positions.length; i++) normals[i] = 0;
  const faces = (indices.length / 3) | 0;
  for (let f = 0; f < faces; f++) {
    const v1 = indices[f * 3] * 3;
    const v2 = indices[f * 3 + 1] * 3;
    const v3 = indices[f * 3 + 2] * 3;
    const ax = positions[v1] - positions[v2];
    const ay = positions[v1 + 1] - positions[v2 + 1];
    const az = positions[v1 + 2] - positions[v2 + 2];
    const bx = positions[v3] - positions[v2];
    const by = positions[v3 + 1] - positions[v2 + 1];
    const bz = positions[v3 + 2] - positions[v2 + 2];
    let nx = ay * bz - az * by;
    let ny = az * bx - ax * bz;
    let nz = ax * by - ay * bx;
    let length = Math.sqrt(nx * nx + ny * ny + nz * nz);
    length = length === 0 ? 1 : length;
    nx /= length;
    ny /= length;
    nz /= length;
    for (const v of [v1, v2, v3]) {
      normals[v] += nx;
      normals[v + 1] += ny;
      normals[v + 2] += nz;
    }
  }
  for (let i = 0; i < normals.length; i += 3) {
    let length = Math.sqrt(normals[i] * normals[i] + normals[i + 1] * normals[i + 1] + normals[i + 2] * normals[i + 2]);
    length = length === 0 ? 1 : length;
    normals[i] /= length;
    normals[i + 1] /= length;
    normals[i + 2] /= length;
  }
}

// ---- the shapes (Babylon's builders) ------------------------------------------------

/** `CreateCylinderVertexData` for `height`, `diameterTop`, `diameterBottom`, `tessellation` (one ring, both caps). */
export function cylinderData(height: number, diameterTop: number, diameterBottom: number, tessellation: number): VData {
  height = height || 2;
  let top = diameterTop === 0 ? 0 : diameterTop || 1;
  let bottom = diameterBottom === 0 ? 0 : diameterBottom || 1;
  top = top || 0.00001;
  bottom = bottom || 0.00001;
  tessellation = (tessellation || 24) | 0;
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  const angleStep = (Math.PI * 2) / tessellation;
  const tan = (bottom - top) / 2 / height;
  const n = [0, 0, 0];
  for (let i = 0; i <= 1; i++) {
    const h = i;
    const radius = (h * (top - bottom) + bottom) / 2;
    for (let j = 0; j <= tessellation; j++) {
      const angle = j * angleStep;
      const x = Math.cos(-angle) * radius;
      const y = -height / 2 + h * height;
      const z = Math.sin(-angle) * radius;
      n[0] = x;
      n[2] = z;
      n[1] = Math.sqrt(x * x + z * z) * tan;
      normalize3(n);
      positions.push(x, y, z);
      normals.push(n[0], n[1], n[2]);
    }
  }
  for (let j = 0; j < tessellation; j++) {
    const i0 = j;
    const i1 = tessellation + 1 + j;
    const i2 = j + 1;
    const i3 = tessellation + 1 + (j + 1);
    indices.push(i0, i1, i2, i3, i2, i1);
  }
  const cap = (isTop: boolean) => {
    const radius = isTop ? top / 2 : bottom / 2;
    if (radius === 0) return;
    const base = positions.length / 3;
    const offset = isTop ? height / 2 : -height / 2;
    positions.push(0, offset, 0);
    normals.push(0, isTop ? 1 : -1, 0);
    for (let i = 0; i <= tessellation; i++) {
      const angle = (Math.PI * 2 * i) / tessellation;
      positions.push(Math.cos(-angle) * radius, offset, Math.sin(-angle) * radius);
      normals.push(0, isTop ? 1 : -1, 0);
    }
    for (let i = 0; i < tessellation; i++) {
      if (isTop) indices.push(base, base + (i + 2), base + (i + 1));
      else indices.push(base, base + (i + 1), base + (i + 2));
    }
  };
  cap(false);
  cap(true);
  return vdata(positions, normals, indices);
}

/** `CreateSphereVertexData` for `diameterX`, `diameterY`, `diameterZ` and `segments` (whole, no slice). */
export function sphereData(diameterX: number, diameterY: number, diameterZ: number, segments: number): VData {
  segments = (segments || 32) | 0;
  const rx = (diameterX || 1) / 2;
  const ry = (diameterY || 1) / 2;
  const rz = (diameterZ || 1) / 2;
  const zSteps = 2 + segments;
  const ySteps = 2 * zSteps;
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  const mz = new Float32Array(16);
  const my = new Float32Array(16);
  const a = [0, 0, 0];
  const c = [0, 0, 0];
  for (let zi = 0; zi <= zSteps; zi++) {
    const angleZ = (zi / zSteps) * Math.PI;
    for (let yi = 0; yi <= ySteps; yi++) {
      const angleY = (yi / ySteps) * Math.PI * 2;
      rotationZ(-angleZ, mz);
      rotationY(angleY, my);
      transformPoint(mz, 0, 1, 0, a);
      transformPoint(my, a[0], a[1], a[2], c);
      positions.push(c[0] * rx, c[1] * ry, c[2] * rz);
      const n = [c[0] / rx, c[1] / ry, c[2] / rz];
      normalize3(n);
      normals.push(n[0], n[1], n[2]);
    }
    if (zi > 0) {
      const count = positions.length / 3;
      for (let first = count - 2 * (ySteps + 1); first + ySteps + 2 < count; first++) {
        indices.push(first, first + 1, first + ySteps + 1, first + ySteps + 1, first + 1, first + ySteps + 2);
      }
    }
  }
  return vdata(positions, normals, indices);
}

/** `CreateTorusVertexData` for `diameter`, `thickness` and `tessellation`. */
export function torusData(diameter: number, thickness: number, tessellation: number): VData {
  diameter = diameter || 1;
  thickness = thickness || 0.5;
  tessellation = (tessellation || 16) | 0;
  const stride = tessellation + 1;
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  const t = new Float32Array(16);
  const r = new Float32Array(16);
  const m = new Float32Array(16);
  const p = [0, 0, 0];
  const n = [0, 0, 0];
  for (let i = 0; i <= tessellation; i++) {
    const outer = (i * Math.PI * 2) / tessellation - Math.PI / 2;
    multiply(translation(diameter / 2, 0, 0, t), rotationY(outer, r), m);
    for (let j = 0; j <= tessellation; j++) {
      const inner = (j * Math.PI * 2) / tessellation + Math.PI;
      const dx = Math.cos(inner);
      const dy = Math.sin(inner);
      transformPoint(m, dx * (thickness / 2), dy * (thickness / 2), 0, p);
      transformNormal(m, dx, dy, 0, n);
      positions.push(p[0], p[1], p[2]);
      normals.push(n[0], n[1], n[2]);
      const ni = (i + 1) % stride;
      const nj = (j + 1) % stride;
      indices.push(i * stride + j, i * stride + nj, ni * stride + j, i * stride + nj, ni * stride + nj, ni * stride + j);
    }
  }
  return vdata(positions, normals, indices);
}

/** `CreateCapsuleVertexData` for `height`, `radius`, `tessellation` and `capSubdivisions` (two subdivisions, upright). */
export function capsuleData(height: number, radius: number, tessellation: number, capSubdivisions: number): VData {
  const heightSegments = 2;
  const radial = Math.max(tessellation || 16, 3) | 0;
  height = Math.max(height || 1, 0);
  radius = Math.max(radius || 0.25, 0);
  const capDetail = Math.max(capSubdivisions || 6, 1) | 0;
  const [rTop, rBottom] = [radius, radius];
  const halfHeight = (height - (rTop + rBottom)) * 0.5;
  const alpha = Math.acos((rBottom - rTop) / height);
  const cosAlpha = Math.cos(alpha);
  const sinAlpha = Math.sin(alpha);
  const pi2 = Math.PI * 0.5;
  const positions: number[] = [];
  const normals: number[] = [];
  const rows: number[][] = [];
  let index = 0;
  const ring = (radiusAt: number, y: number, normal: (sinT: number, cosT: number) => number[]) => {
    const row: number[] = [];
    for (let x = 0; x <= radial; x++) {
      const theta = (x / radial) * 2 * Math.PI;
      const s = Math.sin(theta);
      const c = Math.cos(theta);
      positions.push(radiusAt * s, y, radiusAt * c);
      const n = normal(s, c);
      normals.push(n[0], n[1], n[2]);
      row.push(index++);
    }
    rows.push(row);
  };
  for (let y = 0; y <= capDetail; y++) {
    const a = pi2 - alpha * (y / capDetail);
    const cosA = Math.cos(a);
    const sinA = Math.sin(a);
    ring(cosA * rTop, halfHeight + sinA * rTop, (s, c) => [cosA * s, sinA, cosA * c]);
  }
  const coneHeight = height - rTop - rBottom + cosAlpha * rTop - cosAlpha * rBottom;
  const slope = (sinAlpha * (rBottom - rTop)) / coneHeight;
  for (let y = 1; y <= heightSegments; y++) {
    const r = sinAlpha * ((y * (rBottom - rTop)) / heightSegments + rTop);
    ring(r, halfHeight + cosAlpha * rTop - (y * coneHeight) / heightSegments, (s, c) => normalize3([s, slope, c]));
  }
  for (let y = 1; y <= capDetail; y++) {
    const a = pi2 - alpha - (Math.PI - alpha) * (y / capDetail);
    const cosA = Math.cos(a);
    const sinA = Math.sin(a);
    ring(cosA * rBottom, -halfHeight + sinA * rBottom, (s, c) => [cosA * s, sinA, cosA * c]);
  }
  const indices: number[] = [];
  for (let x = 0; x < radial; x++) {
    for (let y = 0; y < capDetail + heightSegments + capDetail; y++) {
      const i1 = rows[y][x];
      const i2 = rows[y + 1][x];
      const i3 = rows[y + 1][x + 1];
      const i4 = rows[y][x + 1];
      indices.push(i1, i2, i4, i2, i3, i4);
    }
  }
  indices.reverse();
  return vdata(positions, normals, indices);
}

/** `CreateIcoSphereVertexData` for `radius` and `subdivisions`, smooth (`flat: false`). */
export function icoSphereData(radius: number, subdivisions: number): VData {
  radius = radius || 1;
  subdivisions = (subdivisions || 4) | 0;
  const t = (1 + Math.sqrt(5)) / 2;
  const ico = [-1, t, -0, 1, t, 0, -1, -t, 0, 1, -t, 0, 0, -1, -t, 0, 1, -t, 0, -1, t, 0, 1, t, t, 0, 1, t, 0, -1, -t, 0, 1, -t, 0, -1];
  const faces = [
    0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 12, 22, 23, 1, 5, 20, 5, 11, 4, 23, 22, 13, 22, 18, 6, 7, 1, 8, 14, 21, 4, 14, 4, 2, 16, 13, 6, 15, 6, 19, 3, 8, 9, 4, 21, 5, 13, 17, 23, 6, 13, 22, 19, 6, 18,
    9, 8, 1,
  ];
  const unalias = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 2, 3, 3, 3, 4, 7, 8, 9, 9, 10, 11];
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  let current = 0;
  const corner = [0, 1, 2].map(() => [0, 0, 0]);
  const lerp = (a: number[], b: number[], k: number) => [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k];
  for (let face = 0; face < 20; face++) {
    for (let v = 0; v < 3; v++) {
      const id = unalias[faces[3 * face + v]];
      corner[v][0] = ico[3 * id];
      corner[v][1] = ico[3 * id + 1];
      corner[v][2] = ico[3 * id + 2];
      normalize3(corner[v]);
    }
    const vertex = (i1: number, i2: number) => {
      const x0 = lerp(corner[0], corner[2], i2 / subdivisions);
      const x1 = lerp(corner[1], corner[2], i2 / subdivisions);
      // (The corner itself, normalised again in place, as Babylon's builder does.)
      const p = subdivisions === i2 ? corner[2] : lerp(x0, x1, i1 / (subdivisions - i2));
      normalize3(p);
      const n = [p[0] / radius, p[1] / radius, p[2] / radius];
      normalize3(n);
      positions.push(p[0] * radius, p[1] * radius, p[2] * radius);
      normals.push(n[0], n[1], n[2]);
      indices.push(current++);
    };
    for (let i2 = 0; i2 < subdivisions; i2++) {
      for (let i1 = 0; i1 + i2 < subdivisions; i1++) {
        vertex(i1, i2);
        vertex(i1 + 1, i2);
        vertex(i1, i2 + 1);
        if (i1 + i2 + 1 < subdivisions) {
          vertex(i1 + 1, i2);
          vertex(i1 + 1, i2 + 1);
          vertex(i1, i2 + 1);
        }
      }
    }
  }
  return vdata(positions, normals, indices);
}

/**
 * Box with rounded edges and corners (radius `r`), centred on the origin. Each face is a grid
 * whose outer rows are bent around the edge, so normals are smooth across the bevel.
 */
export function roundedBoxData(sx: number, sy: number, sz: number, r: number): VData {
  const half = [sx / 2, sy / 2, sz / 2];
  const radius = Math.max(0, Math.min(r, ...half.map((h) => h * 0.98)));
  const steps = radius > 0 ? 3 : 0;
  // Grid coordinates along one axis: the bevel band is subdivided, the flat middle is one span.
  const axis = (h: number): number[] => {
    if (steps === 0) return [-h, h];
    const inner = h - radius;
    const out: number[] = [];
    for (let i = 0; i <= steps; i++) out.push(-h + (radius * i) / steps);
    for (let i = 0; i <= steps; i++) out.push(inner + (radius * i) / steps);
    return out;
  };
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  const lim = half.map((h) => h - radius);
  // Faces: [normal axis, sign, u axis, v axis]
  const faces: [number, number, number, number][] = [
    [0, 1, 2, 1],
    [0, -1, 2, 1],
    [1, 1, 0, 2],
    [1, -1, 0, 2],
    [2, 1, 0, 1],
    [2, -1, 0, 1],
  ];
  for (const [n, sign, ua, va] of faces) {
    const us = axis(half[ua]);
    const vs = axis(half[va]);
    const base = positions.length / 3;
    for (const v of vs) {
      for (const u of us) {
        const p = [0, 0, 0];
        p[n] = sign * half[n];
        p[ua] = u;
        p[va] = v;
        const inner = [clamp(p[0], -lim[0], lim[0]), clamp(p[1], -lim[1], lim[1]), clamp(p[2], -lim[2], lim[2])];
        const d = [p[0] - inner[0], p[1] - inner[1], p[2] - inner[2]];
        const len = Math.sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
        if (radius > 0 && len > 1e-6) {
          const k = 1 / len;
          const [dx, dy, dz] = [d[0] * k, d[1] * k, d[2] * k];
          positions.push(inner[0] + dx * radius, inner[1] + dy * radius, inner[2] + dz * radius);
          normals.push(dx, dy, dz);
        } else {
          const nn = [0, 0, 0];
          nn[n] = sign;
          positions.push(p[0], p[1], p[2]);
          normals.push(nn[0], nn[1], nn[2]);
        }
      }
    }
    const w = us.length;
    for (let j = 0; j < vs.length - 1; j++) {
      for (let i = 0; i < w - 1; i++) {
        const a = base + j * w + i;
        const b = a + 1;
        const c = a + w;
        const e = c + 1;
        pushTri(positions, normals, indices, a, b, c);
        pushTri(positions, normals, indices, b, e, c);
      }
    }
  }
  return vdata(positions, normals, indices, true);
}

/** Surface of revolution from a profile (see `ExtraShapes.profile`), centred on the origin. */
export function latheData(profile: [number, number][], segments: number): VData {
  // Per profile point: one vertex ring (smooth) or two (hard edge), each with its 2D normal.
  const rings: { r: number; y: number; nr: number; ny: number }[][] = [];
  const segNormal = (a: [number, number], b: [number, number]): [number, number] => {
    const dr = b[0] - a[0];
    const dy = b[1] - a[1];
    const l = Math.hypot(dr, dy) || 1;
    return [dy / l, -dr / l];
  };
  for (let i = 0; i < profile.length; i++) {
    const [r, y] = profile[i];
    const prev = i > 0 ? segNormal(profile[i - 1], profile[i]) : null;
    const next = i < profile.length - 1 ? segNormal(profile[i], profile[i + 1]) : null;
    if (prev && next && prev[0] * next[0] + prev[1] * next[1] > Math.cos((40 * Math.PI) / 180)) {
      const nr = prev[0] + next[0];
      const ny = prev[1] + next[1];
      const l = Math.hypot(nr, ny) || 1;
      rings.push([{ r, y, nr: nr / l, ny: ny / l }]);
    } else {
      const a = prev ?? next!;
      const b = next ?? prev!;
      rings.push([
        { r, y, nr: a[0], ny: a[1] },
        { r, y, nr: b[0], ny: b[1] },
      ]);
    }
  }
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  // Vertex rows: the incoming vertex of point i ends segment i-1, the outgoing one starts segment i.
  const rowStart: { in: number; out: number }[] = [];
  for (const ring of rings) {
    const starts: number[] = [];
    for (const v of ring) {
      starts.push(positions.length / 3);
      for (let k = 0; k <= segments; k++) {
        const a = (k / segments) * Math.PI * 2;
        const c = Math.cos(a);
        const s = Math.sin(a);
        positions.push(v.r * c, v.y, v.r * s);
        normals.push(v.nr * c, v.ny, v.nr * s);
      }
    }
    rowStart.push({ in: starts[0], out: starts[starts.length - 1] });
  }
  for (let i = 0; i < rings.length - 1; i++) {
    const a0 = rowStart[i].out;
    const b0 = rowStart[i + 1].in;
    for (let k = 0; k < segments; k++) {
      pushTri(positions, normals, indices, a0 + k, a0 + k + 1, b0 + k);
      pushTri(positions, normals, indices, a0 + k + 1, b0 + k + 1, b0 + k);
    }
  }
  return vdata(positions, normals, indices, true);
}

/** Lumpy sphere (foliage clumps, rocks): an icosphere displaced by smooth 3D noise. */
export function blobData(sx: number, sy: number, sz: number, amount: number, subdivisions: number, seed: number): VData {
  const data = icoSphereData(0.5, subdivisions);
  // (An updatable mesh's positions, as Babylon builds the blob.)
  const pos = (data.positions = Float32Array.from(data.positions));
  for (let i = 0; i < pos.length; i += 3) {
    const x = pos[i] * 2;
    const y = pos[i + 1] * 2;
    const z = pos[i + 2] * 2;
    const n = noise3(x * 1.7 + seed, y * 1.7, z * 1.7) * 0.7 + noise3(x * 3.9, y * 3.9 + seed, z * 3.9) * 0.3;
    const k = 1 + (n - 0.5) * 2 * amount;
    pos[i] = pos[i] * k * sx;
    pos[i + 1] = pos[i + 1] * k * sy;
    pos[i + 2] = pos[i + 2] * k * sz;
  }
  const normals: number[] = [];
  computeNormals(pos, data.indices, normals);
  // The icosphere repeats vertices along its base faces' edges: average their normals so the
  // lumps shade smoothly instead of in facets.
  const shared = new Map<string, number[]>();
  for (let i = 0; i < pos.length; i += 3) {
    const key = `${pos[i].toFixed(4)},${pos[i + 1].toFixed(4)},${pos[i + 2].toFixed(4)}`;
    const list = shared.get(key);
    if (list) list.push(i);
    else shared.set(key, [i]);
  }
  for (const list of shared.values()) {
    if (list.length < 2) continue;
    let [nx, ny, nz] = [0, 0, 0];
    for (const i of list) [nx, ny, nz] = [nx + normals[i], ny + normals[i + 1], nz + normals[i + 2]];
    const l = Math.hypot(nx, ny, nz) || 1;
    for (const i of list) [normals[i], normals[i + 1], normals[i + 2]] = [nx / l, ny / l, nz / l];
  }
  data.normals = Float64Array.from(normals);
  return data;
}

/**
 * Adds a triangle wound the way front faces are (Babylon's, which `VertexData.ComputeNormals`
 * derives): (a - b) x (c - b) points along the vertex normal.
 */
export function pushTri(pos: ArrayLike<number>, nor: ArrayLike<number>, out: number[], a: number, b: number, c: number): void {
  const ax = pos[a * 3] - pos[b * 3];
  const ay = pos[a * 3 + 1] - pos[b * 3 + 1];
  const az = pos[a * 3 + 2] - pos[b * 3 + 2];
  const cx = pos[c * 3] - pos[b * 3];
  const cy = pos[c * 3 + 1] - pos[b * 3 + 1];
  const cz = pos[c * 3 + 2] - pos[b * 3 + 2];
  const fx = ay * cz - az * cy;
  const fy = az * cx - ax * cz;
  const fz = ax * cy - ay * cx;
  const nx = nor[a * 3] + nor[b * 3] + nor[c * 3];
  const ny = nor[a * 3 + 1] + nor[b * 3 + 1] + nor[c * 3 + 1];
  const nz = nor[a * 3 + 2] + nor[b * 3 + 2] + nor[c * 3 + 2];
  if (fx * nx + fy * ny + fz * nz >= 0) out.push(a, b, c);
  else out.push(a, c, b);
}

/** World-aligned planar UVs (metres) by dominant normal axis, so textures keep their scale. */
export function boxProjectUVs(data: VData): void {
  const pos = data.positions;
  const nor = data.normals;
  if (!nor) return;
  const uvs = pos instanceof Float64Array ? new Float64Array((pos.length / 3) * 2) : new Float32Array((pos.length / 3) * 2);
  for (let i = 0, j = 0; i < pos.length; i += 3, j += 2) {
    const ax = Math.abs(nor[i]);
    const ay = Math.abs(nor[i + 1]);
    const az = Math.abs(nor[i + 2]);
    if (ax >= ay && ax >= az) {
      uvs[j] = pos[i + 2];
      uvs[j + 1] = pos[i + 1];
    } else if (ay >= az) {
      uvs[j] = pos[i];
      uvs[j + 1] = pos[i + 2];
    } else {
      uvs[j] = pos[i];
      uvs[j + 1] = pos[i + 1];
    }
  }
  data.uvs = uvs;
}

// ---- primitive parts ----------------------------------------------------------------

/** A part's turn (`rotate`, degrees about x, y, z) as Babylon's `Quaternion.FromEulerAngles`. */
function partTurn(p: PrimitivePart): Quat {
  const [rx, ry, rz] = (p.rotate ?? [0, 0, 0]).map((d) => (d * Math.PI) / 180);
  return quatFromEuler(rx, ry, rz);
}

/**
 * A primitive part's geometry in model space, sized, turned and placed, with box-projected UVs and
 * its colour (sRGB) per vertex, `alpha` the vertex alpha (the plain finish's roughness and
 * metalness, `encodeFinish`). `lod` 1 is the coarser one for far copies; `index` seeds a blob's lumps.
 */
export function partData(p: PrimitivePart, lod: 0 | 1, index: number, alpha: number): VData {
  const [sx, sy, sz] = p.size;
  const lo = lod === 1;
  let data: VData;
  let scale: Vec3 = [1, 1, 1];
  let shade: ((nx: number, ny: number, nz: number) => number) | null = null;
  // `torus` and `lathe` extend the manifest schema (see `ExtraShapes`).
  const ext = p as PrimitivePart & ExtraShapes;
  switch (p.shape as PrimitivePart['shape'] | 'torus' | 'lathe') {
    case 'torus':
      // size: [outer diameter x, tube thickness, outer diameter z].
      data = torusData(Math.max(sx - sy, 1e-3), sy, p.segments ?? (lo ? 12 : 24));
      scale = [1, 1, sz / sx];
      break;
    case 'lathe':
      data = latheData(ext.profile ?? [[0, -0.5], [0.5, -0.5], [0.5, 0.5], [0, 0.5]], p.segments ?? (lo ? 12 : 24));
      scale = [sx, sy, sz];
      break;
    case 'cylinder':
    case 'cone': {
      const top = p.shape === 'cone' ? (p.taper ?? 0) : (p.taper ?? 1);
      data = cylinderData(sy, top, 1, p.segments ?? (lo ? 8 : p.shape === 'cone' ? 16 : 20));
      scale = [sx, 1, sz];
      break;
    }
    case 'sphere':
      data = sphereData(sx, sy, sz, p.segments ?? (lo ? 6 : 12));
      break;
    case 'capsule':
      data = capsuleData(sy, sx / 2, 16, 6);
      scale = [1, 1, sz / sx];
      break;
    case 'blob':
      data = blobData(sx, sy, sz, p.noise ?? 0.18, p.segments ?? (lo ? 1 : 2), index * 7.31 + sx * 3.7);
      // Fake ambient occlusion: undersides and the inner canopy are darker.
      shade = (_nx, ny) => 0.55 + 0.45 * smoothstep(-0.7, 0.75, ny);
      break;
    default: {
      const r = p.radius ?? Math.min(0.012, Math.min(sx, sy, sz) * 0.25);
      data = roundedBoxData(sx, sy, sz, lo ? 0 : r);
    }
  }
  bakeData(data, compose(scale[0], scale[1], scale[2], p.rotate ? partTurn(p) : [0, 0, 0, 1], p.at[0], p.at[1], p.at[2], new Float32Array(16)));
  boxProjectUVs(data);

  const [cr, cg, cb] = hexColor(p.color);
  const count = data.positions.length / 3;
  const colors = new Float32Array(count * 4);
  const positions = data.positions;
  for (let i = 0; i < count; i++) {
    let k = 1;
    if (shade) {
      // Use the direction from the part centre (not the bumpy normal) for smooth occlusion.
      const dx = (positions[i * 3] - p.at[0]) / (sx / 2);
      const dy = (positions[i * 3 + 1] - p.at[1]) / (sy / 2);
      const dz = (positions[i * 3 + 2] - p.at[2]) / (sz / 2);
      const len = Math.hypot(dx, dy, dz) || 1;
      k = shade(dx / len, dy / len, dz / len);
    }
    colors[i * 4] = cr * k;
    colors[i * 4 + 1] = cg * k;
    colors[i * 4 + 2] = cb * k;
    colors[i * 4 + 3] = alpha;
  }
  data.colors = colors;
  return data;
}

/**
 * A part's exact bounds in model space (`lo`, `hi`: along x, y, z), turned as `partData` turns it:
 * a box by its corners, a cylinder or cone by its rings' corners, a round shape by its own extent (the
 * gate, crates/tools/src/checkglb.rs `bounds`, measures a file's boxes by their corners, turned the same
 * way). And which of its own axes lies along each model axis (-1: none; it's turned off the axes).
 */
export function partBox(p: PrimitivePart): { lo: number[]; hi: number[]; along: number[] } {
  const m = compose(1, 1, 1, partTurn(p), 0, 0, 0, new Float32Array(16));
  // R[j][k]: model axis k of local axis j (row j: where local axis j goes).
  const r = (j: number, k: number) => m[j * 4 + k];
  const along = [0, 1, 2].map((k) => [0, 1, 2].find((j) => Math.abs(r(j, k)) > 0.999) ?? -1);
  const [sx, sy, sz] = p.size;
  const ellipsoid = (k: number, ax: number, ay: number, az: number) => Math.hypot(r(0, k) * ax, r(1, k) * ay, r(2, k) * az);
  const shape = p.shape as PrimitivePart['shape'] | 'torus' | 'lathe';
  let lo: number[];
  let hi: number[];
  if (shape === 'cylinder' || shape === 'cone') {
    // The cylinder's rings: `segments` corners from angle 0, scaled in x and z.
    const n = p.segments ?? (shape === 'cone' ? 16 : 20);
    const top = shape === 'cone' ? (p.taper ?? 0) : (p.taper ?? 1);
    const points: number[][] = [];
    for (const [d, y] of [
      [1, -sy / 2],
      [top, sy / 2],
    ]) {
      for (let j = 0; j < n; j++) points.push([(d / 2) * sx * Math.cos((2 * Math.PI * j) / n), y, (d / 2) * sz * Math.sin((2 * Math.PI * j) / n)]);
    }
    const dot = (q: number[], k: number) => q[0] * r(0, k) + q[1] * r(1, k) + q[2] * r(2, k);
    lo = [0, 1, 2].map((k) => Math.min(...points.map((q) => dot(q, k))));
    hi = [0, 1, 2].map((k) => Math.max(...points.map((q) => dot(q, k))));
  } else if (shape === 'lathe') {
    // Each profile point (radius, y) is a ring, scaled by the size.
    const profile = (p as PrimitivePart & ExtraShapes).profile ?? [[0, -0.5], [0.5, -0.5], [0.5, 0.5], [0, 0.5]];
    lo = [0, 1, 2].map((k) => Math.min(...profile.map(([d, y]) => r(1, k) * y * sy - d * Math.hypot(r(0, k) * sx, r(2, k) * sz))));
    hi = [0, 1, 2].map((k) => Math.max(...profile.map(([d, y]) => r(1, k) * y * sy + d * Math.hypot(r(0, k) * sx, r(2, k) * sz))));
  } else {
    const half = [0, 1, 2].map((k) => {
      if (shape === 'sphere' || shape === 'blob') return ellipsoid(k, sx / 2, sy / 2, sz / 2);
      // `sy` tall overall, radius sx/2, then scaled to sz in z.
      if (shape === 'capsule') return Math.abs(r(1, k)) * Math.max(0, sy / 2 - sx / 2) + ellipsoid(k, sx / 2, sx / 2, sz / 2);
      if (shape === 'torus') {
        // A ring (sx - sy across) swept by a tube sy thick, then scaled to sz in z.
        const [q, ring, tube] = [sz / sx, (sx - sy) / 2, sy / 2];
        return Math.hypot(r(0, k) * ring, r(2, k) * ring * q) + ellipsoid(k, tube, tube, tube * q);
      }
      // A box: model axis k gathers |R[j][k]| * size[j] from each local axis j.
      return [0, 1, 2].reduce((sum, j) => sum + Math.abs(r(j, k)) * p.size[j], 0) / 2;
    });
    lo = half.map((h) => -h);
    hi = half;
  }
  return { lo: p.at.map((c, k) => c + lo[k]), hi: p.at.map((c, k) => c + hi[k]), along };
}

/**
 * A mirror's reflecting face in model space: `matrix` takes the unit square (or, `round`, the
 * unit-diameter disc) in the xy plane, facing +z, onto the face, just in front of the glass.
 * Babylon's matrix layout (a point times the matrix).
 */
export interface MirrorFace {
  round: boolean;
  matrix: Float32Array;
}

/** The reflecting face of a mirror part (see `MirrorFace`), placed as `partData` places the part. */
export function mirrorSurface(p: PrimitivePart): MirrorFace {
  const [sx, sy, sz] = p.size;
  const round = p.shape === 'cylinder';
  // The face's normal before the part turns: a disc's axis (y), a box's thinnest side.
  const axis = round ? 1 : [sx, sy, sz].indexOf(Math.min(sx, sy, sz));
  const turn = partTurn(p);
  const unit = (k: number, v: number): [number, number, number] => [k === 0 ? v : 0, k === 1 ? v : 0, k === 2 ? v : 0];
  const turned = transformPoint(compose(1, 1, 1, turn, 0, 0, 0, new Float32Array(16)), ...unit(axis, 1));
  const sign = (turned[2] >= 0) === ((p.mirror ?? 'front') === 'front') ? 1 : -1;
  // Unit square / disc in xy facing +z -> sized, onto the face on `axis` (a hair in front), turned and placed.
  const [w, h] = axis === 0 ? [sz, sy] : axis === 1 ? [sx, sz] : [sx, sy];
  const offset = unit(axis, (p.size[axis] / 2 + 0.002) * sign);
  const m = new Float32Array(16);
  multiply(scaling(w, h, 1, new Float32Array(16)), faceTurn(axis, sign), m);
  multiply(m, translation(offset[0], offset[1], offset[2], new Float32Array(16)), m);
  multiply(m, compose(1, 1, 1, turn, p.at[0], p.at[1], p.at[2], new Float32Array(16)), m);
  return { round, matrix: m };
}

/** Turns the unit face (facing +z) to face `sign` along `axis`. */
function faceTurn(axis: number, sign: number): Float32Array {
  const m = new Float32Array(16);
  if (axis === 0) return rotationY((sign * Math.PI) / 2, m) as Float32Array;
  if (axis === 1) return rotationX((-sign * Math.PI) / 2, m) as Float32Array;
  return (sign > 0 ? identity(m) : rotationY(Math.PI, m)) as Float32Array;
}

/**
 * The reflecting face of a file's mirror glass (`MaterialOverride.mirror`), as `mirrorSurface` of a
 * primitive part: across the glass's thinnest side, a hair in front of the face towards `side`
 * (front: +z, or +x / +y for glass turned that way). Round if its triangles facing that way cover
 * well under its box (a disc covers π/4 of its square).
 */
export function fileMirror(data: VData, { lo, hi }: { lo: number[]; hi: number[] }, side: 'front' | 'back'): MirrorFace {
  const size = [0, 1, 2].map((k) => hi[k] - lo[k]);
  const axis = size.indexOf(Math.min(...size));
  const sign = side === 'front' ? 1 : -1;
  const [w, h] = axis === 0 ? [size[2], size[1]] : axis === 1 ? [size[0], size[2]] : [size[0], size[1]];
  // How much of the box the face covers: the triangles facing `side`, measured across `axis`.
  const pos = data.positions;
  const nor = data.normals ?? new Float32Array(pos.length);
  const idx = data.indices;
  const [u, v] = [0, 1, 2].filter((k) => k !== axis);
  let area = 0;
  for (let t = 0; t + 2 < idx.length; t += 3) {
    const [a, b, c] = [idx[t] * 3, idx[t + 1] * 3, idx[t + 2] * 3];
    if ((nor[a + axis] + nor[b + axis] + nor[c + axis]) * sign <= 0) continue;
    area += Math.abs((pos[b + u] - pos[a + u]) * (pos[c + v] - pos[a + v]) - (pos[c + u] - pos[a + u]) * (pos[b + v] - pos[a + v])) / 2;
  }
  const round = w > 0 && h > 0 && area / (w * h) < 0.9;
  const centre = [(lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2];
  centre[axis] += (size[axis] / 2 + 0.002) * sign;
  const m = new Float32Array(16);
  multiply(scaling(w, h, 1, new Float32Array(16)), faceTurn(axis, sign), m);
  multiply(m, translation(centre[0], centre[1], centre[2], new Float32Array(16)), m);
  return { round, matrix: m };
}

// ---- resizing and fitting -----------------------------------------------------------

/** Share of a model's height above which everything rises with it (seats, tops, backs; sim-core `Top::of`). */
const RISE_FROM = 0.4;
/** Along the floor: the band at each edge (m, at most this share of half the width) that moves with the edge unstretched. */
const EDGE_BAND = 0.35;
const EDGE_SHARE = 0.4;

/** Whether `resize` changes a model of `from` tiles. */
export function resizes(from: readonly [number, number], resize: Resize): boolean {
  return resize.size[0] !== from[0] || resize.size[1] !== from[1] || !!resize.tall;
}

/**
 * The resizing of a model of `from` tiles whose height runs from `bottom` to `top`, as a map per
 * axis (x, y, z) of where a point goes, and its slope there (how much the axis stretches).
 */
export function resizeMaps(from: readonly [number, number], resize: Resize, bottom: number, top: number): { maps: ((v: number) => number)[]; slopes: ((v: number) => number)[]; span: number } {
  const [tw, td] = resize.size;
  const dy = resize.tall / 100;
  const span = Math.max(top - bottom, 1e-3);
  // Upward (or, from the ceiling, downward): what lies past the line moves by `dy`.
  const line = resize.ceiling ? top - RISE_FROM * span : bottom + RISE_FROM * span;
  const lift = (v: number) => (resize.ceiling ? (v < line ? v - dy : v) : v > line ? v + dy : v);
  // Along the floor: a band at each edge moves with it; the middle scales to fill.
  const across = (half: number, half2: number) => {
    const band = Math.min(EDGE_BAND, EDGE_SHARE * half);
    const band2 = Math.min(band, EDGE_SHARE * half2);
    const inner = half - band;
    const inner2 = half2 - band2;
    const map = (v: number) => {
      const a = Math.abs(v);
      const mapped = a <= inner ? (a * inner2) / Math.max(inner, 1e-6) : inner2 + ((a - inner) * band2) / band;
      return Math.sign(v) * mapped;
    };
    const slope = (v: number) => (Math.abs(v) <= inner ? inner2 / Math.max(inner, 1e-6) : band2 / band);
    return { map, slope };
  };
  const [x, z] = [across(from[0] / 2, tw / 2), across(from[1] / 2, td / 2)];
  return { maps: [x.map, lift, z.map], slopes: [x.slope, () => 1, z.slope], span };
}

/**
 * Geometry resized by `resizeMaps`, point by point: the edges move out keeping their thickness, the
 * middle stretches, and what lies above the rise line rises (a triangle across it stretches).
 * Normals follow the stretch.
 */
export function resizeData(data: { positions: ArrayLike<number> & { [i: number]: number }; normals?: (ArrayLike<number> & { [i: number]: number }) | null }, maps: ((v: number) => number)[], slopes: ((v: number) => number)[]): void {
  const pos = data.positions;
  const nor = data.normals;
  for (let i = 0; i < pos.length; i += 3) {
    const [x, y, z] = [pos[i], pos[i + 1], pos[i + 2]];
    if (nor) {
      // A normal scales by the inverse of the stretch.
      const n = [nor[i] / slopes[0](x), nor[i + 1] / slopes[1](y), nor[i + 2] / slopes[2](z)];
      const l = Math.hypot(n[0], n[1], n[2]) || 1;
      for (let k = 0; k < 3; k++) nor[i + k] = n[k] / l;
    }
    pos[i] = maps[0](x);
    pos[i + 1] = maps[1](y);
    pos[i + 2] = maps[2](z);
  }
}

/** The box (`lo`, `hi`: corners along x, y, z) of `positions`. */
export function dataBox(positions: ArrayLike<number>): { lo: number[]; hi: number[] } {
  const lo = [Infinity, Infinity, Infinity];
  const hi = [-Infinity, -Infinity, -Infinity];
  for (let i = 0; i < positions.length; i += 3) {
    for (let k = 0; k < 3; k++) {
      lo[k] = Math.min(lo[k], positions[i + k]);
      hi[k] = Math.max(hi[k], positions[i + k]);
    }
  }
  return { lo, hi };
}

/** Scales and moves geometry so its joint bounds fill `size`, bottom on y = 0; the matrix it did that with. */
export function fitInto(pieces: VData[], [w, h, d]: Vec3, align: 'centre' | 'back', [ox, oy, oz]: Vec3): Float32Array {
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (const data of pieces) {
    const { lo, hi } = dataBox(data.positions);
    for (let k = 0; k < 3; k++) {
      min[k] = Math.min(min[k], lo[k]);
      max[k] = Math.max(max[k], hi[k]);
    }
  }
  const sx = w / Math.max(max[0] - min[0], 1e-6);
  const sy = h / Math.max(max[1] - min[1], 1e-6);
  const sz = d / Math.max(max[2] - min[2], 1e-6);
  const cx = (min[0] + max[0]) / 2;
  const cz = align === 'back' ? min[2] : (min[2] + max[2]) / 2;
  const tz = align === 'back' ? -0.5 + oz : oz;
  const m = new Float32Array(16);
  multiply(scaling(sx, sy, sz, new Float32Array(16)), translation(ox - cx * sx, oy - min[1] * sy, tz - cz * sz, new Float32Array(16)), m);
  for (const data of pieces) transformData(data, m);
  return m;
}

/**
 * Sets `data`'s normals to `local` (a node's own) taken to model space by `world`'s inverse
 * transpose, as a normal goes (a part scaled to nothing at rest: its normals lean towards the grow
 * axis, and the instance's pose turns them back into the grown shape's).
 */
export function restNormals(data: VData, local: ArrayLike<number> | null | undefined, world: Readonly<Mat4>): void {
  if (!local || !data.normals) return;
  const m = transpose(invert(world, new Float32Array(16)));
  const v = [0, 0, 0];
  const normals = Float32Array.from(local);
  for (let i = 0; i < normals.length; i += 3) {
    transformNormal(m, normals[i], normals[i + 1], normals[i + 2], v);
    normalize3(v);
    normals[i] = v[0];
    normals[i + 1] = v[1];
    normals[i + 2] = v[2];
  }
  data.normals = normals;
}

/**
 * A copy of a file's geometry as a primitive part (positions, normals, metre UVs, vertex colour
 * carrying the finish), so it merges with primitive parts of the same finish: in `ov`'s colour, or
 * `base` (the file material's, sRGB) without one, times the shading the file bakes into its vertex
 * colours. `full`, the same geometry where a part grown from nothing is grown, gives the normals,
 * winding and UVs its flat rest shape can't (a textured finish's normal map draws nothing on UVs
 * collapsed along an axis).
 */
export function restyledData(data: VData, base: readonly [number, number, number], ov: MaterialOverride, alpha: number, full?: VData): VData {
  const shape = full ?? data;
  const positions = shape.positions;
  const indices = data.indices;
  let normals = shape.normals;
  if (!normals) {
    normals = new Float32Array(positions.length);
    computeNormals(positions, indices, normals);
  }
  // The source winding depends on the file's handedness and the baked transform; re-wind every
  // triangle to match its vertex normals the way the finish materials expect (see `pushTri`).
  const wound: number[] = [];
  for (let i = 0; i + 2 < indices.length; i += 3) pushTri(positions, normals, wound, indices[i], indices[i + 1], indices[i + 2]);
  const out: VData = { positions: Float32Array.from(positions), normals: Float32Array.from(normals), indices: Uint32Array.from(wound) };
  boxProjectUVs(out);
  if (full) {
    out.positions = Float32Array.from(data.positions);
    // (Normals at rest that the instance's pose turns into the grown shape's: `restNormals`.)
    if (data.normals) out.normals = Float32Array.from(data.normals);
  }
  const [r, g, b] = ov.color ? hexColor(ov.color) : base;
  const count = positions.length / 3;
  const colors = new Float32Array(count * 4);
  // The file's grey vertex colour is the shading baked into its creases (gamekit/shade.py), linear: the
  // colour here is sRGB (the finish shader raises it to 2.2), so it takes the shading's 1/2.2 power.
  const baked = data.colors && data.colors.length >= count * 4 ? data.colors : null;
  for (let i = 0; i < count; i++) {
    const k = baked ? Math.max(0, baked[i * 4]) ** (1 / 2.2) : 1;
    colors.set([r * k, g * k, b * k, alpha], i * 4);
  }
  out.colors = colors;
  return out;
}

/** The glowing faces (`MaterialOverride.glow`) as one mesh, each piece in its glow colour (linear). */
export function glowData(pieces: { data: VData; color: string }[]): VData {
  const positions: number[] = [];
  const normals: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];
  for (const { data, color } of pieces) {
    const base = positions.length / 3;
    const pos = data.positions;
    const c = hexColor(color).map((v) => Math.pow(v, 2.2));
    positions.push(...pos);
    normals.push(...(data.normals ?? new Float32Array(pos.length)));
    for (let i = 0; i < pos.length / 3; i++) colors.push(c[0], c[1], c[2], 1);
    for (const i of data.indices) indices.push(base + i);
  }
  return { positions: Float32Array.from(positions), normals: Float32Array.from(normals), colors: Float32Array.from(colors), indices: Uint32Array.from(indices) };
}

// ---- small math helpers -------------------------------------------------------------

/** Babylon's `Color3.FromHexString` (sRGB 0..1; black for anything but `#rrggbb`). */
export function hexColor(hex: string): [number, number, number] {
  if (hex.substring(0, 1) !== '#' || hex.length !== 7) return [0, 0, 0];
  return [parseInt(hex.substring(1, 3), 16) / 255, parseInt(hex.substring(3, 5), 16) / 255, parseInt(hex.substring(5, 7), 16) / 255];
}

/** Babylon's `Vector3.normalize`, in place. */
function normalize3(v: number[]): number[] {
  const len = Math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
  if (len === 0 || len === 1) return v;
  const k = 1 / len;
  v[0] *= k;
  v[1] *= k;
  v[2] *= k;
  return v;
}

/** Babylon's `Matrix.determinant`. */
export function determinant(m: Readonly<Mat4>): number {
  const [m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33] = m;
  const d2233 = m22 * m33 - m32 * m23;
  const d2133 = m21 * m33 - m31 * m23;
  const d2132 = m21 * m32 - m31 * m22;
  const d2033 = m20 * m33 - m30 * m23;
  const d2032 = m20 * m32 - m22 * m30;
  const d2031 = m20 * m31 - m30 * m21;
  const c00 = +(m11 * d2233 - m12 * d2133 + m13 * d2132);
  const c01 = -(m10 * d2233 - m12 * d2033 + m13 * d2032);
  const c02 = +(m10 * d2133 - m11 * d2033 + m13 * d2031);
  const c03 = -(m10 * d2132 - m11 * d2032 + m12 * d2031);
  return m00 * c00 + m01 * c01 + m02 * c02 + m03 * c03;
}

/** `m` transposed, in place. */
function transpose(m: Mat4): Mat4 {
  for (let i = 0; i < 4; i++) {
    for (let j = i + 1; j < 4; j++) {
      const t = m[i * 4 + j];
      m[i * 4 + j] = m[j * 4 + i];
      m[j * 4 + i] = t;
    }
  }
  return m;
}

export function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

function smoothstep(a: number, b: number, x: number): number {
  const t = clamp((x - a) / (b - a), 0, 1);
  return t * t * (3 - 2 * t);
}

function hash3(x: number, y: number, z: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(y, 668265263) + Math.imul(z, 1440662683)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Smooth value noise in [0, 1]. */
function noise3(x: number, y: number, z: number): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const zi = Math.floor(z);
  const fx = x - xi;
  const fy = y - yi;
  const fz = z - zi;
  const u = fx * fx * (3 - 2 * fx);
  const v = fy * fy * (3 - 2 * fy);
  const w = fz * fz * (3 - 2 * fz);
  const l = (a: number, b: number, t: number) => a + (b - a) * t;
  const c = (dx: number, dy: number, dz: number) => hash3(xi + dx, yi + dy, zi + dz);
  return l(l(l(c(0, 0, 0), c(1, 0, 0), u), l(c(0, 1, 0), c(1, 1, 0), u), v), l(l(c(0, 0, 1), c(1, 0, 1), u), l(c(0, 1, 1), c(1, 1, 1), u), v), w);
}
