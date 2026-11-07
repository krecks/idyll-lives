/**
 * The traffic lights' signal heads as the snapshot has them (sim-core `traffic::Signal`, the
 * snapshot's `light` rows: one per arm of each lit junction, the same rows from tick to tick): where
 * each pole stands on the kerb, which way its head faces (a resident's yaw: towards those coming in
 * along its arm) and which lamp it shows. Read into arrays made once (`SignalRows.read`), so the
 * renderer (`pc/signals.ts`) reads them every frame without allocating, and knows whether the poles
 * moved (a new world) or only the lamps changed.
 */

import type { SnapshotLayout } from '../../core/snapshot';

/** The lamp a head shows: its lens, top to bottom (`SIGNAL_LAMPS`). */
export const RED = 0;
export const AMBER = 1;
export const GREEN = 2;
export type Lamp = typeof RED | typeof AMBER | typeof GREEN;

/** The lenses top to bottom, as the model's lens materials are named (`lens_<name>`, tools/art/blender/families/street_furniture.py). */
export const SIGNAL_LAMPS = ['red', 'amber', 'green'] as const;

/** Signal heads read at most (the snapshot's room, sim-core `snapshot::MAX_LIGHTS`, when the layout doesn't say). */
export const MOST_SIGNALS = 256;

/** What changed in the rows since the last read. */
export interface SignalChange {
  /** Heads came or went, or one stands elsewhere (a new world). */
  placed: boolean;
  /** A head shows another lamp. */
  lit: boolean;
}

/** The heads to be seen in the newest snapshot (rows not shown are left out), read again each frame. */
export class SignalRows {
  /** Heads read. */
  count = 0;
  readonly x: Float32Array;
  readonly z: Float32Array;
  readonly yaw: Float32Array;
  /** Each head's `Lamp`. */
  readonly lamp: Uint8Array;
  private readonly change: SignalChange = { placed: false, lit: false };

  constructor(readonly most = MOST_SIGNALS) {
    this.x = new Float32Array(most);
    this.z = new Float32Array(most);
    this.yaw = new Float32Array(most);
    this.lamp = new Uint8Array(most);
  }

  /**
   * The heads in `snap` (laid out as `layout`; an older layout without them: none). What changed
   * since the last read (the same object each time: read it before the next).
   */
  read(snap: ArrayLike<number>, layout: SnapshotLayout): SignalChange {
    const change = this.change;
    change.placed = false;
    change.lit = false;
    const f = layout.light;
    const base = layout.lightBase;
    const stride = layout.lightStride;
    const at = layout.header.lightCount;
    const room = Math.min(this.most, layout.maxLights ?? MOST_SIGNALS);
    const rows = !f || base === undefined || stride === undefined || at === undefined ? 0 : Math.max(0, Math.min(room, snap[at] | 0));
    let n = 0;
    for (let k = 0; k < rows; k++) {
      const o = base! + k * stride!;
      if (!snap[o + f!.shown]) continue;
      const x = snap[o + f!.x];
      const z = snap[o + f!.z];
      const yaw = snap[o + f!.yaw];
      const lamp = lampOf(snap[o + f!.state], f!.states);
      // (Compared as stored: a Float32Array's own values.)
      if (n >= this.count || this.x[n] !== Math.fround(x) || this.z[n] !== Math.fround(z) || this.yaw[n] !== Math.fround(yaw)) change.placed = true;
      else if (this.lamp[n] !== lamp) change.lit = true;
      this.x[n] = x;
      this.z[n] = z;
      this.yaw[n] = yaw;
      this.lamp[n] = lamp;
      n++;
    }
    if (n !== this.count) change.placed = true;
    this.count = n;
    // (Placed again, every head's lamp is drawn again with it.)
    if (change.placed) change.lit = true;
    return change;
  }

  /** None read (a new world: the next read places them all). */
  clear(): void {
    this.count = 0;
  }
}

/**
 * The lamp a row's `state` shows: amber or green as they are; red otherwise, walk among them (red
 * for traffic while those on foot cross) and a value this renderer doesn't know.
 */
export function lampOf(state: number, states: { red: number; amber: number; green: number; walk: number }): Lamp {
  if (state === states.green) return GREEN;
  if (state === states.amber) return AMBER;
  return RED;
}
