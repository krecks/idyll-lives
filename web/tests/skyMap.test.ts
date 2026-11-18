/**
 * The sky's dome as the haze reads it (`render/shared/skyMap.ts`, `render/pc/shaders/fog.ts` `gameSkyAt`): each of the
 * dome's vertices (`sphereData`, the sky's `SKY_SEGMENTS`) found where it is from its direction as the shaders have it
 * (PlayCanvas space: the dome's, game space, mirrored in z), its colour worked out there as the dome's own, and read
 * between the four round a direction as the dome's colours lie between its vertices.
 */
import { describe, expect, test } from 'bun:test';
import { sphereData } from '../src/render/shared/modelParts';
import { domeColour, SKY_COLUMNS, SKY_ROWS, SKY_SEGMENTS, skyCell, skyColourAt, skyVertex, type DomeSky } from '../src/render/shared/skyMap';

const { positions } = sphereData(2, 2, 2, SKY_SEGMENTS);
/** A golden hour's sky (linear colours), the sun low in the west, and the same in PlayCanvas space. */
const game: DomeSky = { low: [0.66, 0.7, 0.77], high: [0.42, 0.55, 0.78], sun: [0.8, 0.12, -0.59], glow: [0.9, 0.6, 0.35] };
const engine: DomeSky = { ...game, sun: [game.sun[0], game.sun[1], -game.sun[2]] };

describe("the sky's dome as the haze reads it", () => {
  test("each vertex's direction falls on its own place, and is the dome's vertex there", () => {
    expect(positions.length / 3).toBe(SKY_ROWS * SKY_COLUMNS);
    for (let row = 1; row < SKY_ROWS - 1; row++) {
      for (let col = 0; col < SKY_COLUMNS; col++) {
        const k = (row * SKY_COLUMNS + col) * 3;
        // (Game space to PlayCanvas's: z mirrored.)
        const [x, y, z] = [positions[k], positions[k + 1], -positions[k + 2]];
        const [c, r] = skyCell(x, y, z);
        expect(r).toBeCloseTo(row, 6);
        // (The last column is the first again.)
        expect((col === SKY_COLUMNS - 1 ? [0, SKY_COLUMNS - 1] : [col]).some((w) => Math.abs(c - w) < 1e-6)).toBe(true);
        const v = skyVertex(col, row, [0, 0, 0]);
        for (const [a, b] of [[v[0], x], [v[1], y], [v[2], z]]) expect(a).toBeCloseTo(b, 6);
      }
    }
    expect(skyCell(0, 1, 0)[1]).toBe(0);
    expect(skyCell(0, -1, 0)[1]).toBeCloseTo(SKY_ROWS - 1, 9);
  });

  test("its colour at a vertex is the dome's there, and between them read between the four round it", () => {
    for (const k of [SKY_COLUMNS * 17 + 5, SKY_COLUMNS * 16 + 40, SKY_COLUMNS * 3 + 67, SKY_COLUMNS * 20 + 10]) {
      const [x, y, z] = [positions[k * 3], positions[k * 3 + 1], positions[k * 3 + 2]];
      // (The dome's colour of its vertex in its own space, game space; the haze's in PlayCanvas's.)
      const dome = domeColour(game, x, y, z, [0, 0, 0]);
      const haze = skyColourAt(engine, x, y, -z);
      for (let c = 0; c < 3; c++) expect(haze[c]).toBeCloseTo(dome[c], 6);
    }
    // Halfway between two vertices along a row: halfway between their colours.
    const [a, b] = [skyVertex(10, 17, [0, 0, 0]), skyVertex(11, 17, [0, 0, 0])];
    const mid = skyColourAt(engine, ...skyVertex(10.5, 17, [0, 0, 0]));
    const [ca, cb] = [domeColour(engine, ...a, [0, 0, 0]), domeColour(engine, ...b, [0, 0, 0])];
    for (let c = 0; c < 3; c++) expect(mid[c]).toBeCloseTo((ca[c] + cb[c]) / 2, 6);
  });

  test("below the horizon's band, away from the sun's glow, it is the haze's own colour", () => {
    // (The haze's colour as PlayCanvas takes it: the horizon's linearised twice.)
    const [r, g, b] = skyColourAt(engine, -0.6, -0.4, 0.4);
    for (const [have, want] of [[r, game.low[0]], [g, game.low[1]], [b, game.low[2]]]) expect(have).toBeCloseTo(Math.pow(Math.pow(want, 2.2), 2.2), 6);
  });
});
