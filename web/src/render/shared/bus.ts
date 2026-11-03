/**
 * The town's bus as the game draws it (sim-core `bus.rs`): a single-decker town bus, longer and
 * taller than the cars, made of the street's plain-finish primitives as they are (`paintBus`):
 * a two-tone body on six-stud wheels, a band of big windows, a deep windscreen with the line's
 * number lit on the sign above it, and two doors on its kerb side (its right: -x, front towards
 * +z), each of two glass leaves that slide apart to open (`busLeaf`, `BUS_DOORS`).
 *
 * Engine-neutral: the moving buses (`pc/buses.ts`) draw it.
 */

import type { V3 } from './geometry';
import type { Painter } from './street';

/** Its size (m: sim-core `bus::BUS_LEN`, `BUS_WID`), height, ground clearance and wheels (radius; how far its axles are from its middle). */
export const BUS = { len: 8.6, wid: 2.3, height: 2.95, floor: 0.34, wheelR: 0.46, axles: [2.35, -2.2] as const };

/** Its doors (sim-core `bus::DOOR_ALONG`: the front one, then the middle one): their middle along it (m ahead of its middle) and width. On its right (-x). */
export const BUS_DOORS: readonly { z: number; w: number }[] = [
  { z: 3.25, w: 1.05 },
  { z: -0.55, w: 1.25 },
];

/** How high a door's opening is, from the ground. */
const DOOR_TOP = 2.32;
/** The body's livery: below the windows, the band between and the roof; glass; the dark inside of a door. */
const BODY = '#23707F';
const BAND = '#F1ECDD';
const ROOF = '#E6E1D3';
const GLASS = '#22313B';
const INSIDE = '#7A7F84';
const TRIM = '#1B1E21';
/** A door's frame (brushed metal) and the yellow grab poles in an open doorway. */
const FRAME = '#B8BDC1';
const POLE = '#E8B321';
/** The band of windows: from the belt to the top (m). */
const BELT = 1.12;
const TOP = 2.36;

/**
 * The bus in `p`'s frame (front towards +z, on the ground at `y`), without its wheels (drawn on
 * their own to turn) and with its doors shut, or (`open`) without them: their openings dark, the
 * leaves drawn on their own (`busLeaf`) to slide. `line`: the number on its sign (1 to 99).
 */
export function paintBus(p: Painter, line: number, open = false): void {
  const { len, wid, height, floor, wheelR } = BUS;
  const [hw, hl] = [wid / 2, len / 2];
  const zf = hl;
  const zb = -hl;
  // The lower body: a skirt from the floor to the belt, rounded a little at the corners.
  p.paint(BODY, 0.35, 0);
  p.box(-hw, floor, zb + 0.12, hw, BELT, zf - 0.12, 'ny');
  p.box(-hw + 0.08, floor, zb, hw - 0.08, BELT, zb + 0.12, 'ny');
  p.box(-hw + 0.08, floor, zf - 0.12, hw - 0.08, BELT, zf, 'ny');
  // The window band and the roof, a little in from the sides; the cream band round its top.
  p.paint(GLASS, 0.06, 0).box(-hw + 0.03, BELT, zb + 0.15, hw - 0.03, TOP, zf - 0.2, 'ny py nz pz');
  p.paint(BAND, 0.4, 0).box(-hw, TOP, zb + 0.08, hw, height - 0.12, zf - 0.1, 'ny');
  p.paint(ROOF, 0.5, 0).box(-hw + 0.12, height - 0.12, zb + 0.3, hw - 0.12, height, zf - 0.4, 'ny');
  // Pillars between the side windows (none across the doors), a strip along the belt.
  p.paint(BODY, 0.35, 0);
  for (const side of [-1, 1]) {
    const x = side * (hw - 0.01);
    const xo = side * (hw + 0.012);
    for (let z = zb + 0.6; z < zf - 0.4; z += 1.45) {
      if (side < 0 && BUS_DOORS.some((d) => Math.abs(z - d.z) < d.w / 2 + 0.12)) continue;
      p.box(Math.min(x, xo), BELT, z - 0.07, Math.max(x, xo), TOP, z + 0.07, 'ny py');
    }
    p.paint(BAND, 0.4, 0).box(Math.min(x, xo), BELT - 0.06, zb + 0.15, Math.max(x, xo), BELT, zf - 0.2, 'ny');
    p.paint(BODY, 0.35, 0);
  }
  // The doors: shut, glass leaves in a dark frame; open, the dark inside and its step.
  for (const d of BUS_DOORS) {
    const [z0, z1] = [d.z - d.w / 2, d.z + d.w / 2];
    const x = -hw - 0.006;
    if (open) {
      // The doorway (the inside showing on the side, the leaves slid away): its step, its frame,
      // a yellow grab pole either side.
      p.paint(INSIDE, 0.85, 0).poly([[x, floor + 0.04, z0], [x, DOOR_TOP, z0], [x, DOOR_TOP, z1], [x, floor + 0.04, z1]], [-1, 0, 0]);
      p.paint('#55595D', 0.85, 0).box(-hw - 0.1, floor - 0.08, z0 + 0.02, -hw, floor + 0.04, z1 - 0.02, 'ny');
      p.paint(TRIM, 0.6, 0);
      for (const z of [z0, z1]) p.box(-hw - 0.02, floor, z - 0.035, -hw + 0.002, DOOR_TOP, z + 0.035, 'ny');
      p.box(-hw - 0.02, DOOR_TOP - 0.05, z0, -hw + 0.002, DOOR_TOP + 0.02, z1, 'ny');
      p.paint(POLE, 0.35, 0);
      for (const z of [z0 + 0.14, z1 - 0.14]) p.tube('y', floor + 0.05, DOOR_TOP - 0.08, -hw - 0.04, z, 0.025, 0.025, 6);
    } else {
      leafAt(p, x, z0, z1);
      leafAt(p, x, (z0 + z1) / 2, z1);
    }
  }
  // The front: a deep windscreen under the sign, the face and its lamps, the bumper.
  const fz = zf + 0.004;
  p.paint(BAND, 0.4, 0).poly([[-hw + 0.08, floor, fz], [hw - 0.08, floor, fz], [hw - 0.08, height - 0.12, fz], [-hw + 0.08, height - 0.12, fz]], [0, 0, 1]);
  const glass = (q: V3[], n: V3) => p.paint(GLASS, 0.06, 0).poly(q, n);
  glass([[-hw + 0.16, 0.98, fz + 0.004], [hw - 0.16, 0.98, fz + 0.004], [hw - 0.16, 2.42, fz + 0.004], [-hw + 0.16, 2.42, fz + 0.004]], [0, 0, 1]);
  p.paint(TRIM, 0.5, 0).box(-0.025, 0.98, fz, 0.025, 2.42, fz + 0.012, 'ny');
  // The sign: dark, the number lit amber.
  p.paint('#121416', 0.6, 0).box(-hw + 0.3, 2.5, fz - 0.01, hw - 0.3, 2.76, fz + 0.012, 'ny');
  digits(p, Math.max(1, Math.min(99, Math.round(line))), 0.62, 2.63, fz + 0.016);
  p.paint(BODY, 0.35, 0).box(-hw + 0.08, floor, fz - 0.01, hw - 0.08, 0.92, fz + 0.01, 'ny');
  p.paint('#F5F3EA', 0.1, 0);
  for (const s of [-1, 1]) p.box(s * (hw - 0.42), 0.62, fz, s * (hw - 0.14), 0.78, fz + 0.02, 'ny');
  p.paint('#2A2C2E', 0.6, 0).box(-hw + 0.04, floor - 0.06, zf - 0.06, hw - 0.04, floor + 0.2, zf + 0.06);
  // The back: the body round a smaller window, its lamps, the bumper.
  const bz = zb - 0.004;
  p.paint(BODY, 0.35, 0).poly([[-hw + 0.08, BELT, bz + 0.002], [hw - 0.08, BELT, bz + 0.002], [hw - 0.08, TOP, bz + 0.002], [-hw + 0.08, TOP, bz + 0.002]], [0, 0, -1]);
  glass([[-hw + 0.22, 1.35, bz], [hw - 0.22, 1.35, bz], [hw - 0.22, 2.3, bz], [-hw + 0.22, 2.3, bz]], [0, 0, -1]);
  p.paint('#A3161E', 0.2, 0);
  for (const s of [-1, 1]) p.box(s * (hw - 0.3), 0.6, bz - 0.02, s * (hw - 0.1), 1.0, bz, 'ny');
  p.paint('#2A2C2E', 0.6, 0).box(-hw + 0.04, floor - 0.06, zb - 0.06, hw - 0.04, floor + 0.2, zb + 0.06);
  // Mirrors on stalks ahead of the windscreen.
  p.paint(TRIM, 0.5, 0);
  for (const s of [-1, 1]) {
    p.box(s * hw - 0.02, 2.05, zf - 0.05, s * (hw + 0.22) + 0.02, 2.09, zf + 0.25);
    p.box(s * (hw + 0.17), 1.72, zf + 0.2, s * (hw + 0.27), 2.08, zf + 0.3);
  }
  // Dark wheel arches (the wheels go in them).
  for (const az of BUS.axles) {
    for (const side of [-1, 1]) {
      p.paint('#141517', 0.9, 0).tube('x', side * (hw + 0.002), side * (hw + 0.004), wheelR, az, wheelR + 0.09, wheelR + 0.09, 10, true, 0, Math.PI);
    }
  }
}

/** A door's glass leaf on the side plane at `x`, from `z0` to `z1`: a metal frame, the glass in it, a rubber seal down its edge. */
function leafAt(p: Painter, x: number, z0: number, z1: number): void {
  const { floor } = BUS;
  p.paint(FRAME, 0.35, 0.6).poly([[x, floor + 0.04, z0], [x, DOOR_TOP, z0], [x, DOOR_TOP, z1], [x, floor + 0.04, z1]], [-1, 0, 0]);
  p.paint('#3C5662', 0.05, 0).poly([[x - 0.004, floor + 0.32, z0 + 0.07], [x - 0.004, DOOR_TOP - 0.1, z0 + 0.07], [x - 0.004, DOOR_TOP - 0.1, z1 - 0.07], [x - 0.004, floor + 0.32, z1 - 0.07]], [-1, 0, 0]);
  p.paint(TRIM, 0.7, 0).poly([[x - 0.006, floor + 0.04, z1 - 0.025], [x - 0.006, DOOR_TOP, z1 - 0.025], [x - 0.006, DOOR_TOP, z1], [x - 0.006, floor + 0.04, z1]], [-1, 0, 0]);
}

/** Half of door `d` (`BUS_DOORS`): a glass leaf `w` wide in `p`'s frame, its middle at the origin on the bus's side, shut. It slides open along it (`pc/buses.ts`). */
export function busLeaf(p: Painter, d: number): void {
  const w = BUS_DOORS[d].w / 2;
  const x = -0.004;
  leafAt(p, x, -w / 2, w / 2);
  // Its inside, facing in.
  p.paint(INSIDE, 0.9, 0).poly([[x + 0.03, BUS.floor + 0.04, -w / 2], [x + 0.03, DOOR_TOP, -w / 2], [x + 0.03, DOOR_TOP, w / 2], [x + 0.03, BUS.floor + 0.04, w / 2]], [1, 0, 0]);
}

/** Where wheel `w` of the bus is: 0 front right, 1 front left, 2 back right, 3 back left (+1: +x, its left); the front ones steer. */
export function busWheelAt(w: number): { side: 1 | -1; x: number; z: number; front: boolean } {
  const side = w % 2 === 0 ? -1 : 1;
  const front = w < 2;
  return { side, x: side * (BUS.wid / 2 + 0.012), z: front ? BUS.axles[0] : BUS.axles[1], front };
}

/** The segments lit for each digit (a b c d e f g: top, top right, bottom right, bottom, bottom left, top left, middle). */
const SEGMENTS = ['abcdef', 'bc', 'abged', 'abgcd', 'fgbc', 'afgcd', 'afgcde', 'abc', 'abcdefg', 'abcdfg'];

/** `n` (1 to 99) in lit amber segments on the sign, its middle at (`cx`, `cy`), `w` wide at most, facing +z at `z`. */
function digits(p: Painter, n: number, w: number, cy: number, z: number): void {
  const text = String(n);
  const [dw, dh, t] = [Math.min(0.13, w / text.length - 0.05), 0.17, 0.022];
  const gap = 0.05;
  const all = text.length * dw + (text.length - 1) * gap;
  p.paint('#FFB21E', 0.3, 0);
  [...text].forEach((ch, k) => {
    const x0 = all / 2 - k * (dw + gap) - dw;
    const [l, r] = [x0, x0 + dw];
    const [b, m, top] = [cy - dh / 2, cy, cy + dh / 2];
    const seg: Record<string, [number, number, number, number]> = {
      a: [l, top - t, r, top],
      b: [l, m, l + t, top],
      c: [l, b, l + t, m],
      d: [l, b, r, b + t],
      e: [r - t, b, r, m],
      f: [r - t, m, r, top],
      g: [l, m - t / 2, r, m + t / 2],
    };
    // (Seen from the front, +x is the sign's right: `b` and `c` on its +x side read the right way round.)
    for (const s of SEGMENTS[Number(ch)] ?? '') {
      const [x0s, y0, x1s, y1] = seg[s];
      p.box(-x1s, y0, z, -x0s, y1, z + 0.004, 'ny');
    }
  });
}
