import type { FrameState } from '../../../core/bridge';

export const CURL = ':curl';
/**
 * A curled-up sleeper's quilt (`quiltGeometry`), a throw over them on a sofa: how far (m) it reaches past the feet,
 * how far its sides hang down (just over the seat cushion's front edge, not to the floor), and how far past the
 * neck its top edge is, towards the head (the shoulders and the arm over them under it, the head out), lying over
 * the shoulder rather than tucked down.
 */
export const CURL_QUILT = { past: 0.06, drop: 0.14, neck: -0.06 };
/**
 * Where someone the newest snapshot has sat at (x, z), facing `yaw`, sits in a car that drives (or,
 * `aboard`: rode in last frame, till it has come to a stop), as the car is drawn this frame (`out`: x,
 * z, yaw); false when they aren't in one (`pc/cars.ts` `ride`).
 */
export type CarRide = (frame: FrameState, x: number, z: number, yaw: number, aboard: boolean, out: { x: number; z: number; yaw: number }) => boolean;

/** Where a bike or an e-scooter left out by a place stands (its middle) and which way it faces (rad, as a placement's turn). */
export interface ParkedAt {
  x: number;
  z: number;
  yaw: number;
}

/** How fast an upper-layer clip fades in over the one it replaces (per second; a quarter of a second, less when the
 * action runs fast: scaled by `animRate` like the lower fade). */
export const UPPER_FADE_RATE = 4;

/** How far ahead of the feet (m, body scale) the work clips' hands are at a counter (`clips/work.ts` `COUNTER_Z`, leaning in). */
export const STATION_REACH = 0.5;
/** The most (m) a cook steps in to a station's tool (the body clear of a cooker's front: `stationReach`). */
export const STATION_STEP_MAX = 0.45;
/**
 * The most (m) a cook steps along a station to its tool (`stationReach`): along a station only they can use (one slot),
 * to its far end; where two can cook side by side, within their own half.
 */
export const STATION_ALONG_MAX = 1.2;
export const STATION_ALONG_SHARED = 0.7;
/** How far (m) in from the ends of a station's top a tool placed in front of the cook stays (`toolSpot`). */
export const TOOL_END_MARGIN = 0.2;

/**
 * Residents far off (`faceDetail`'s `poseFar`: `POSE_FAR_PX`) are posed in full every this many frames, not every frame
 * (staggered by row, so as many each frame): every second frame, every fourth with their faces at half that size. Between, the pose goes along with them as it is (`carryPose`). And never more than `FAR_LAG` s apart.
 */
export const FAR_EVERY = [2, 4] as const;
export const FAR_LAG = 0.25;
/**
 * How long a resident the sim is catching up is hidden (real ms, `setDetailArea`): come into its full-detail area
 * (the camera's new area reaches the sim's worker, and its next tick catches them up on the ticks they owe, a few
 * frames), or seen to jump further than they could have gone (a catch-up out of it, or after a pause).
 */
export const REVEAL_MS = 250;
/** A step in the land under a resident (m) past which it is eased, not taken at once, and the easing's time constant (s: most of the way in 120 ms). */
export const GROUND_STEP = 0.05;
export const GROUND_EASE = 0.04;

/** How fast a rise cut short (`SimRig.riseK`) finishes its step off the seat (per second: a quarter of a second, less
 * when the action runs fast). */
export const RISE_CATCH_UP = 4;
