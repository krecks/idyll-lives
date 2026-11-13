/**
 * Which lamps cast shadows (engine-neutral; `pc/lampShadows.ts` draws them): the few nearest what the camera
 * looks at, each keeping its place until another is clearly nearer, so the choice doesn't flip while the camera
 * rests between two lamps.
 */

/**
 * Marks in `want` the `count` lamps nearest the camera's target: `distance[i]` is lamp i's (m; `Infinity` for one
 * that can't cast now), counted as `keep` of itself (under 1) for a lamp that casts already (`casting[i]`).
 * Returns whether any is wanted. No allocations.
 */
export function pickCasters(distance: ArrayLike<number>, casting: ArrayLike<boolean>, count: number, keep: number, want: boolean[]): boolean {
  const n = distance.length;
  for (let i = 0; i < n; i++) want[i] = false;
  const reach = (i: number) => (casting[i] ? distance[i] * keep : distance[i]);
  let any = false;
  for (let k = 0; k < count; k++) {
    let best = -1;
    for (let i = 0; i < n; i++) {
      if (want[i] || !(distance[i] < Infinity)) continue;
      if (best < 0 || reach(i) < reach(best)) best = i;
    }
    if (best < 0) break;
    want[best] = true;
    any = true;
  }
  return any;
}
