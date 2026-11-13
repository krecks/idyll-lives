/** The town tags fade in with `townBlend`: from 0.2, fully at 0.5. */
export const TAGS_FROM = 0.2;
export const TAGS_FULL = 0.5;

/** How far the tags have faded in at `blend` (0 up to `TAGS_FROM`, 1 from `TAGS_FULL`, smoothstep between). */
export function tagFade(blend: number): number {
  const t = Math.min(1, Math.max(0, (blend - TAGS_FROM) / (TAGS_FULL - TAGS_FROM)));
  return t * t * (3 - 2 * t);
}

/**
 * The household's own tags come in with the town view itself, as its buildings do: from its start, fully by a blend
 * of 0.03 (about 150 m out), so the family and its home are marked wherever the town view is in.
 */
export const MINE_FULL = 0.03;
/** The household's tags are on screen (and take clicks) from this much of their fade. */
export const MINE_SEEN = 0.25;

/** How far the household's tags have faded in at `blend` (0 at 0, 1 from `MINE_FULL`). */
export function mineFade(blend: number): number {
  return Math.min(1, Math.max(0, blend / MINE_FULL));
}
