/**
 * The render resolution (Settings › Performance › Resolution): how many lines of pixels the game draws,
 * 720p, 1080p (the default), 1440p or 2160p, the width following the window's shape. Never more than the
 * screen has: on a 1080p screen 1440p and 2160p draw its own 1080 lines, and a window smaller than the
 * setting is drawn at its own size. Fewer lines than the screen has are scaled up by the browser; the
 * interface is drawn at the screen's own resolution either way.
 *
 * `renderScale` is pure (no DOM, no engine), so it can be checked on its own.
 */

/** The resolutions offered: lines of pixels drawn. */
export const RESOLUTIONS = [720, 1080, 1440, 2160] as const;
export type Resolution = (typeof RESOLUTIONS)[number];
/** The resolution a new player starts with. */
export const DEFAULT_RESOLUTION: Resolution = 1080;

export function isResolution(value: unknown): value is Resolution {
  return RESOLUTIONS.includes(value as Resolution);
}

/**
 * The share of the canvas's device pixels to draw along each side, for a canvas `height` device pixels
 * tall: the setting's lines, at most every line the canvas has (1).
 */
export function renderScale(height: number, resolution: Resolution): number {
  if (!(height > 0)) return 1;
  const lines = isResolution(resolution) ? resolution : DEFAULT_RESOLUTION;
  return Math.min(1, lines / height);
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
