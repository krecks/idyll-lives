/**
 * The render resolution (Settings › Performance › Resolution). `auto` draws every pixel of the
 * screen up to a budget of about 1920×1080: a 1080p screen is drawn at 100 %, a Retina or 4K one at
 * about 2.07 million pixels, which the browser scales up (the picture stays sharp: the scene is
 * smoothed and the interface is drawn at the screen's own resolution). A number draws that share of
 * the screen's resolution along each side (0.5..1), whatever the screen.
 *
 * `renderScale` is pure (no DOM, no Babylon), so it can be checked on its own.
 */

/** `auto`, or a share of native resolution along each side (`MIN_SCALE`..1). */
export type Resolution = 'auto' | number;

/** Pixels `auto` draws at most: 1920×1080. */
export const PIXEL_BUDGET = 1920 * 1080;
/** The lowest share the resolution setting offers. */
export const MIN_SCALE = 0.5;

/** The share of native resolution along each side to draw at, for a canvas of `width`×`height` device pixels. */
export function renderScale(width: number, height: number, resolution: Resolution): number {
  if (resolution !== 'auto') return Number.isFinite(resolution) ? Math.min(1, Math.max(MIN_SCALE, resolution)) : 1;
  const pixels = width * height;
  return pixels > PIXEL_BUDGET ? Math.sqrt(PIXEL_BUDGET / pixels) : 1;
}

/**
 * Calls `onChange` whenever the device pixel ratio changes: browser zoom, or the window moving to a
 * screen with another pixel density (which doesn't always resize the window). Returns the unsubscribe.
 */
export function watchPixelRatio(onChange: () => void): () => void {
  let query: MediaQueryList | null = null;
  const listen = () => {
    query?.removeEventListener('change', changed);
    query = matchMedia(`(resolution: ${window.devicePixelRatio || 1}dppx)`);
    query.addEventListener('change', changed);
  };
  // (A query matches one ratio: once it stops matching, watch the new one.)
  const changed = () => {
    listen();
    onChange();
  };
  listen();
  return () => query?.removeEventListener('change', changed);
}
