/**
 * How loud a sound is to the ear, to level one against another by: a measure for the tools and the tests
 * (`tools/audio/listen.ts`, `web/tests/sounds.test.ts`), not something the game runs.
 */

/** `x` through a biquad (`b` over 1, `a[0]`, `a[1]`), in double precision. */
function biquad(x: Float64Array, b: readonly [number, number, number], a: readonly [number, number]): Float64Array {
  const y = new Float64Array(x.length);
  let x1 = 0;
  let x2 = 0;
  let y1 = 0;
  let y2 = 0;
  for (let i = 0; i < x.length; i++) {
    const v = b[0] * x[i] + b[1] * x1 + b[2] * x2 - a[0] * y1 - a[1] * y2;
    x2 = x1;
    x1 = x[i];
    y2 = y1;
    y1 = v;
    y[i] = v;
  }
  return y;
}

/**
 * How loud a sound is to the ear (LUFS, ITU-R BS.1770-4): both channels through the K weighting (a shelf that
 * lifts what the head lifts, then a high-pass: the standard's coefficients, which are for 48 kHz), the mean
 * square of 0.4 s blocks a tenth of a second apart, the blocks under -70 LUFS and then those more than 10 LU under
 * the rest's mean left out (a sound's own silences don't make it quieter). A sound shorter than a block is one
 * block. Peak and RMS say what the samples hold; this says what one sound is against another when heard, which
 * is what a mix is levelled by.
 */
export function loudness(l: Float32Array, r: Float32Array, rate: number): number {
  if (rate !== 48_000) throw new Error(`loudness is measured at 48 kHz (the K weighting's coefficients are for it), not ${rate}`);
  const weighted = (x: Float32Array) =>
    biquad(biquad(Float64Array.from(x), [1.53512485958697, -2.69169618940638, 1.19839281085285], [-1.69065929318241, 0.73248077421585]), [1, -2, 1], [-1.99004745483398, 0.99007225036621]);
  const a = weighted(l);
  const b = weighted(r);
  const n = Math.min(a.length, b.length);
  const block = Math.min(n, Math.round(0.4 * rate));
  const step = Math.round(0.1 * rate);
  if (block === 0) return -Infinity;
  const powers: number[] = [];
  for (let start = 0; start + block <= n; start += step) {
    let sum = 0;
    for (let i = start; i < start + block; i++) sum += a[i] * a[i] + b[i] * b[i];
    powers.push(sum / block);
  }
  const lufs = (power: number) => (power > 0 ? -0.691 + 10 * Math.log10(power) : -Infinity);
  const mean = (list: number[]) => list.reduce((s, v) => s + v, 0) / Math.max(1, list.length);
  const heard = powers.filter((p) => lufs(p) > -70);
  if (!heard.length) return -Infinity;
  const floor = lufs(mean(heard)) - 10;
  return lufs(mean(heard.filter((p) => lufs(p) > floor)));
}
