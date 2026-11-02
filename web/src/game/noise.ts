/**
 * Noise for laying out land: a lattice hash, value noise and fractal noise over it, and a smooth
 * step. The same functions the renderer's landscape uses (`render/shared/nature.ts`, which is to
 * re-export these: note for the renderer branch), so the world's generator and its drawing agree.
 */

/** A number 0..1 for a lattice point and a seed. */
export function hash(x: number, z: number, seed: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(z, 668265263) + Math.imul(seed, 2147483647)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Value noise, 0..1; with a `period`, its lattice wraps round every `period` cells (a tileable noise). */
export function valueNoise(x: number, z: number, seed: number, period = 0): number {
  const xi = Math.floor(x);
  const zi = Math.floor(z);
  const xf = x - xi;
  const zf = z - zi;
  const u = xf * xf * (3 - 2 * xf);
  const v = zf * zf * (3 - 2 * zf);
  const [x0, z0] = period ? [((xi % period) + period) % period, ((zi % period) + period) % period] : [xi, zi];
  const [x1, z1] = period ? [(x0 + 1) % period, (z0 + 1) % period] : [xi + 1, zi + 1];
  const a = hash(x0, z0, seed);
  const b = hash(x1, z0, seed);
  const c = hash(x0, z1, seed);
  const d = hash(x1, z1, seed);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}

/** Fractal noise: `octaves` of `valueNoise`, each twice as fine and half as strong (0 .. 1 − 0.5^octaves); with a `period` (cells of the first), tileable. */
export function fbm(x: number, z: number, seed: number, octaves = 4, period = 0): number {
  let sum = 0;
  let amp = 0.5;
  let freq = 1;
  for (let i = 0; i < octaves; i++) {
    sum += amp * valueNoise(x * freq, z * freq, seed + i * 17, period * freq);
    amp *= 0.5;
    freq *= 2;
  }
  return sum;
}

export const smoothstep = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};
