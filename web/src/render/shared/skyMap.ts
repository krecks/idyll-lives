/**
 * The sky's dome as the haze reads it (`pc/shaders/fog.ts` `gameSkyAt`): the dome's colour (`pc/nature.ts` `Sky`) worked
 * out at its four vertices round a direction (`sphereData`'s: rows from the top down, each round from +x) and read
 * between them as its triangles have it between them, from the four colours it is made of (`DomeSky`: uniforms, not a
 * map, so its programs read no more textures: a ground's on WebGL2 is at its 16 already). Where the land's veil is
 * whole a thing takes the colour the dome has behind it. Kept apart from the engine for the tests.
 */

import { smoothstep } from '../../game/noise';

/** The dome's sphere's segments (`sphereData`). */
export const SKY_SEGMENTS = 32;
/** The dome's rows of vertices, top to bottom, and the vertices round each (its first again at its end). */
export const SKY_ROWS = SKY_SEGMENTS + 3;
export const SKY_COLUMNS = 2 * (SKY_SEGMENTS + 2) + 1;

/** The dome's colour goes from `low` at the horizon to `high` over this band of the direction's height (its sine). */
export const SKY_BAND: readonly [number, number] = [0.02, 0.75];
/** The glow round the sun: these shares of its colour at these powers of the cosine from it, from this height up. */
export const SKY_GLOW: readonly [number, number, number, number] = [0.55, 6, 0.6, 48];
export const GLOW_FROM = -0.05;

/** What the dome's colour is made of: the horizon's and the zenith's (linear), the way to the sun, and its glow's colour (its strength in it). */
export interface DomeSky {
  low: [number, number, number];
  high: [number, number, number];
  sun: [number, number, number];
  glow: [number, number, number];
}

/**
 * The dome's vertex colour for a direction (`dx`, `dy`, `dz`, a unit vector in the space `sky.sun` is in), into `out`:
 * the horizon's colour to the zenith's, and the glow round the sun, linearised twice (`Sky`: the look's colours as it
 * was tuned with them).
 */
export function domeColour(sky: DomeSky, dx: number, dy: number, dz: number, out: [number, number, number]): [number, number, number] {
  const t = smoothstep(SKY_BAND[0], SKY_BAND[1], dy);
  const d = Math.max(0, dx * sky.sun[0] + dy * sky.sun[1] + dz * sky.sun[2]);
  const g = dy > GLOW_FROM ? Math.pow(d, SKY_GLOW[1]) * SKY_GLOW[0] + Math.pow(d, SKY_GLOW[3]) * SKY_GLOW[2] : 0;
  for (let c = 0; c < 3; c++) out[c] = Math.pow(Math.pow(sky.low[c] + (sky.high[c] - sky.low[c]) * t + sky.glow[c] * g, 2.2), 2.2);
  return out;
}

/**
 * Where a direction (`dx`, `dy`, `dz`, PlayCanvas space: the dome's own, game space, mirrored in z) falls among the
 * dome's vertices (into `out`): its column, from 0 to `SKY_COLUMNS` - 1, and its row, from 0 (up) to `SKY_ROWS` - 1
 * (down), between whole ones read between the vertices. As the shaders work it out.
 */
export function skyCell(dx: number, dy: number, dz: number, out: [number, number] = [0, 0]): [number, number] {
  const l = Math.hypot(dx, dy, dz) || 1;
  // (A vertex's direction in the dome's space: (sin a cos b, cos a, -sin a sin b), a down from the top, b round.)
  const row = (Math.acos(Math.min(1, Math.max(-1, dy / l))) * (SKY_ROWS - 1)) / Math.PI;
  let col = (Math.atan2(dz, dx) * (SKY_COLUMNS - 1)) / (2 * Math.PI);
  if (col < 0) col += SKY_COLUMNS - 1;
  out[0] = col;
  out[1] = row;
  return out;
}

/** The direction (PlayCanvas space) of the dome's vertex at `col`, `row` (whole), into `out`. */
export function skyVertex(col: number, row: number, out: [number, number, number]): [number, number, number] {
  const [a, b] = [(row * Math.PI) / (SKY_ROWS - 1), (col * 2 * Math.PI) / (SKY_COLUMNS - 1)];
  out[0] = Math.sin(a) * Math.cos(b);
  out[1] = Math.cos(a);
  out[2] = Math.sin(a) * Math.sin(b);
  return out;
}

/**
 * The dome's colour behind a direction (PlayCanvas space, `sky.sun` in it too) as `gameSkyAt` works it out: its four
 * vertices' colours round it (`domeColour`), read between them. For the tests.
 */
export function skyColourAt(sky: DomeSky, dx: number, dy: number, dz: number): [number, number, number] {
  const [col, row] = skyCell(dx, dy, dz);
  const i = Math.min(Math.floor(col), SKY_COLUMNS - 2);
  const j = Math.min(Math.floor(row), SKY_ROWS - 2);
  const [u, v] = [col - i, row - j];
  const at = (a: number, b: number) => domeColour(sky, ...skyVertex(a, b, [0, 0, 0]), [0, 0, 0]);
  const [p, q, r, s] = [at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1)];
  return [0, 1, 2].map((c) => (p[c] * (1 - u) + q[c] * u) * (1 - v) + (r[c] * (1 - u) + s[c] * u) * v) as [number, number, number];
}
