// The Babylon.js 9.30 fixtures (`tests/fixtures/babylon/*.json`): what Babylon's own code gave for the
// tests' inputs, recorded by `record-babylon.ts` while the game still had a Babylon renderer. Long runs
// of numbers are kept as the bytes of a Float32Array or Float64Array in base64 (exact, and a tenth of
// the size of the numbers written out); short ones as JSON numbers, which hold a double exactly too.
import { readFileSync } from 'node:fs';

/** A recorded run of numbers: `{ f32 }` or `{ f64 }` (base64 of the array's bytes), or a plain array. */
export type Numbers = { f32: string } | { f64: string } | { u32: string } | number[];

export function f32(a: ArrayLike<number>): Numbers {
  return { f32: Buffer.from(Float32Array.from(a).buffer).toString('base64') };
}
export function f64(a: ArrayLike<number>): Numbers {
  return { f64: Buffer.from(Float64Array.from(a).buffer).toString('base64') };
}
export function u32(a: ArrayLike<number>): Numbers {
  return { u32: Buffer.from(Uint32Array.from(a).buffer).toString('base64') };
}

/** The numbers back: a Float32Array, Float64Array or Uint32Array as recorded (a plain array as it is). */
export function numbers(n: Numbers): ArrayLike<number> & Iterable<number> {
  if (Array.isArray(n)) return n;
  // (A copy, so the array has its own, aligned buffer.)
  const bytes = (s: string) => new Uint8Array(Buffer.from(s, 'base64')).buffer;
  if ('f32' in n) return new Float32Array(bytes(n.f32));
  if ('f64' in n) return new Float64Array(bytes(n.f64));
  return new Uint32Array(bytes(n.u32));
}

/** A fixture by name (`tests/fixtures/babylon/<name>.json`). */
export function fixture<T>(name: string): T {
  return JSON.parse(readFileSync(new URL(`./babylon/${name}.json`, import.meta.url), 'utf8')) as T;
}
