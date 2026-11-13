/**
 * The town view's shared state: what the renderer, the town-scale drawing and the HUD's
 * town tags read and write each frame. Engine-neutral: no PlayCanvas, no Svelte.
 *
 * The renderer (camera stream) writes `blend`, `radius`, `target` and `project` every frame and sets `goTo` once; the
 * town-scale drawing and the tags only read them.
 */
// pin:living-town-view begin
/** Where to fly the camera: a town point, optionally a radius to end at, and a resident to select and follow. */
export interface TownGoTo {
  x: number;
  z: number;
  radius?: number;
  select?: number;
}

export interface TownViewState {
  /** `townBlend(radius)`: 0 in the play view, 1 in the full town view. */
  blend: number;
  /** The orbit's radius (m). */
  radius: number;
  /** The orbit's target (town metres). */
  target: { x: number; z: number };
  /** A world point to CSS pixels on the canvas (`[x, y, depth m]`), or null behind the camera or off screen. */
  project: ((x: number, y: number, z: number) => [number, number, number] | null) | null;
  /** Fly there (`shared/focus.ts` `Flight`, `cameraRig.ts`); null until the renderer is up. */
  goTo: ((to: TownGoTo) => void) | null;
}

export const townView: TownViewState = { blend: 0, radius: 0, target: { x: 0, z: 0 }, project: null, goTo: null };
// pin:living-town-view end
