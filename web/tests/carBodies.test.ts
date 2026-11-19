/**
 * The bodies households' cars come in (`shared/cars.ts` `CARS`): the ones content names (`careers.json` `carBodies`,
 * which the sim parks and drives by their size) in the same order and the same size, drawn within that size, their
 * side profiles convex, their doors on their sides between the wheels' arches and the cab's ends; and the bodies
 * content's careers ask for all there.
 */
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { CARS, carDoor, DOORS, FAMILY_CARS, paintCar } from '../src/render/shared/cars';
import { Geo } from '../src/render/shared/geometry';
import { Painter } from '../src/render/shared/street';

type Careers = {
  carBodies: { id: string; label: string; len: number; wid: number }[];
  grades: { id: string; carAge?: string }[];
  careerCategories: { id: string; car?: Record<string, string>; tracks: { id: string; car?: Record<string, string> }[] }[];
};
const careers = JSON.parse(readFileSync(new URL('../public/content/careers.json', import.meta.url), 'utf8')) as Careers;

describe('the cars’ bodies', () => {
  test('are content’s, in its order and of its size', () => {
    expect(CARS.map((b) => b.id)).toEqual(careers.carBodies.map((b) => b.id));
    for (const [k, b] of careers.carBodies.entries()) expect([CARS[k].len, CARS[k].wid]).toEqual([b.len, b.wid]);
    // (The town's own pick among the first, the family cars, as before there were more.)
    expect(CARS.slice(0, FAMILY_CARS).map((b) => b.id)).toEqual(['saloon', 'hatchback', 'suv', 'estate']);
  });

  test('every field’s and track’s car is a body, from a grade there is', () => {
    const bodies = new Set(careers.carBodies.map((b) => b.id));
    const grades = new Set(careers.grades.map((g) => g.id));
    for (const c of careers.careerCategories) {
      expect(c.car).toBeDefined();
      for (const map of [c.car, ...c.tracks.map((t) => t.car)]) {
        for (const [g, b] of Object.entries(map ?? {})) {
          expect(grades.has(g)).toBe(true);
          expect(bodies.has(b)).toBe(true);
        }
      }
    }
    for (const g of careers.grades) expect([undefined, 'old', 'used', 'new']).toContain(g.carAge);
  });

  test('are drawn within their size, on the ground', () => {
    for (const body of CARS) {
      for (const age of [0, 1, 2] as const) {
        const geo = new Geo();
        paintCar(new Painter(geo).at(0, 0), body, '#808080', false, true, age);
        const p = geo.meshData().positions;
        let [x0, x1, y0, z0, z1] = [Infinity, -Infinity, Infinity, Infinity, -Infinity];
        for (let i = 0; i < p.length; i += 3) {
          [x0, x1, y0] = [Math.min(x0, p[i]), Math.max(x1, p[i]), Math.min(y0, p[i + 1])];
          [z0, z1] = [Math.min(z0, p[i + 2]), Math.max(z1, p[i + 2])];
        }
        // (Its mirrors stand out of its width, as a real car's do: 0.12 m each side.)
        expect(x1 - x0).toBeLessThanOrEqual(body.wid + 2 * 0.12 + 0.05);
        // (Its bumpers and plates out of its length, as the family cars' always were: 0.2 m at most.)
        expect(z1 - z0).toBeLessThanOrEqual(body.len + 0.2);
        expect(z1 - z0).toBeGreaterThan(body.len - 0.15);
        // (The wheels' dark arches reach a little under the ground, out of sight, as they always did.)
        expect(y0).toBeGreaterThanOrEqual(-0.071);
      }
    }
  });

  test('have convex side profiles and their doors on their sides', () => {
    for (const body of CARS) {
      const low = body.low;
      for (let i = 0; i < low.length; i++) {
        const [a, b, c] = [low[i], low[(i + 1) % low.length], low[(i + 2) % low.length]];
        const cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        expect(cross).toBeGreaterThan(0);
      }
      const [br, bf] = body.cabin;
      for (let d = 0; d < DOORS; d++) {
        const door = carDoor(body, d);
        // Hinged at their front edge, their back edge behind it, within the cab's length.
        expect(door.hinge).toBeGreaterThan(door.rearBelt);
        expect(door.hinge).toBeGreaterThan(door.rearLow);
        expect(door.hinge).toBeLessThanOrEqual(bf + 1e-6);
        expect(door.rearBelt).toBeGreaterThanOrEqual(br - 1e-6);
        expect(door.sill).toBeLessThan(body.cabin[4]);
      }
    }
  });
});
