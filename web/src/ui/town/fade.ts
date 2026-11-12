/** The town tags fade in with `townBlend` (docs/design/living-town.md, Town tags): from 0.2, fully at 0.5. */
export const TAGS_FROM = 0.2;
export const TAGS_FULL = 0.5;

/** How far the tags have faded in at `blend` (0 up to `TAGS_FROM`, 1 from `TAGS_FULL`, smoothstep between). */
export function tagFade(blend: number): number {
  const t = Math.min(1, Math.max(0, (blend - TAGS_FROM) / (TAGS_FULL - TAGS_FROM)));
  return t * t * (3 - 2 * t);
}
