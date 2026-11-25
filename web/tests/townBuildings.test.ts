// The town view's buildings (`render/shared/townBuildings.ts`): a lot's building closed (`lotLookBuilding`, a named
// place's or a flats building's) and a far lot's house fitted to the unit meshes, their lights by the hour, and the
// unit meshes' faces facing out.
import { describe, expect, test } from 'bun:test';
import type { LotLook } from '../src/core/protocol';
import { homeLights, litAt, lotBuilding, lotLookBuilding, MOST_STOREYS, PALETTE, ROOFS, STYLE, unitMeshes } from '../src/render/shared/townBuildings';

const flat = () => 0;
const look = (use: LotLook['use'], roof: LotLook['roof'] = 'gable', n = 3): LotLook => ({ use, storeys: 2, roof, look: n });
/** A far lot's house's one box (`HousePlanner.farLook`'s part): its walls to `to` m, its roof `rise` m over them. */
const part = (box: [number, number, number, number], to: number, rise: number) => ({ box, from: 0, to, rise });

describe("a lot's building closed", () => {
  test('its walls\' box exactly, along its longer side, from under the ground to its real storeys', () => {
    // A 12 × 8 m house's box on a pad 2 m up, the land 1.5 m round it.
    const t = lotLookBuilding([94, 46, 106, 54], look('house'), 2, 2, () => 1.5);
    expect([t.cx, t.cz, t.w, t.d, t.ux, t.uz]).toEqual([100, 50, 12, 8, 1, 0]);
    // (Its floor its pad, its walls from under its lowest ground, two storeys of 3 m, a 34° roof over 4 m.)
    expect(t.floor).toBe(2);
    expect(t.base).toBeLessThan(1.5);
    expect(t.top).toBe(8);
    expect(t.rise).toBeCloseTo(4 * Math.tan((34 * Math.PI) / 180), 6);
    // Longer along z: along z, a hip's ridge too.
    const hip = lotLookBuilding([0, 0, 6, 14], look('flats', 'hip', PALETTE.length * 2 + 1), 3, 0, flat);
    expect([hip.w, hip.d, hip.ux, hip.uz]).toEqual([14, 6, 0, 1]);
    const roof = lotLookBuilding([0, 0, 6, 14], look('flats', 'flat'), 3, 0, flat);
    expect(roof.rise).toBeLessThan(1);
    expect(roof.top).toBe(9);
  });

  test('never taller than its real storeys, whatever its look says, and at most the world\'s three', () => {
    // A look of two storeys over a building of one: one; of four storeys: three.
    expect(lotLookBuilding([0, 0, 10, 8], { ...look('shop'), storeys: 3 }, 1, 0, flat).top).toBe(3);
    expect(lotLookBuilding([0, 0, 10, 8], look('office', 'flat'), 4, 0, flat).top).toBe(MOST_STOREYS * 3);
    expect(lotLookBuilding([0, 0, 10, 8], look('office', 'flat'), 0, 0, flat).top).toBe(3);
  });

  test('the same footprint the same colours, wherever its lot is listed; a pitched roof toned down from its palette', () => {
    const a = lotLookBuilding([0, 0, 10, 6], look('house', 'gable', 0), 1, 0, flat);
    expect(lotLookBuilding([0, 0, 10, 6], look('house', 'gable', 0), 1, 0, flat)).toEqual(a);
    expect(lotLookBuilding([20, 0, 30, 6], look('house', 'gable', 0), 1, 0, flat).wall).not.toEqual(a.wall);
    const palette = parseInt(PALETTE[0][1].slice(1, 3), 16) / 255;
    expect(a.paint!.roof[0]).toBeLessThan(palette * 0.85);
    expect(a.paint!.roof[0]).toBeGreaterThan(palette * 0.6);
    // (Far off, as the roof's texture shows it: darker than painted.)
    expect(a.roofColor[0]).toBeLessThan(a.paint!.roof[0]);
  });
});

describe("a far lot's house", () => {
  test('keeps its own look and box', () => {
    const [t] = lotBuilding({ id: 3, chunk: 0, box: [10, 20, 18, 30], storeys: 2, wall: '#ECE6D8', roof: '#5C626A', shape: 'hip', parts: [part([10, 20, 18, 30], 6, 2.4)], pad: 0 }, flat);
    expect([t.cx, t.cz, t.w, t.d, t.top, t.rise, t.roof]).toEqual([14, 25, 10, 8, 6, 2.4, 'hip']);
    expect(t.uz).toBe(1);
  });

  test('on a hill stands on its pad, its walls from under its lowest corner', () => {
    // (The land rising 0.5 m a metre in x: its corners from 5 to 9 m; the pad at 7.2 m.)
    const slope = (x: number) => x * 0.5;
    const [t] = lotBuilding({ id: 3, chunk: 0, box: [10, 20, 18, 30], storeys: 2, wall: '#ECE6D8', roof: '#5C626A', shape: 'hip', parts: [part([10, 20, 18, 30], 6, 2.4)], pad: 7.2 }, slope);
    expect(t.floor).toBe(7.2);
    expect(t.top).toBeCloseTo(13.2, 6);
    expect(t.base).toBeCloseTo(5 - 0.4, 6);
    // A pad under the lowest corner (cut into the hill): the walls from under the pad.
    const [cut] = lotBuilding({ id: 3, chunk: 0, box: [10, 20, 18, 30], storeys: 1, wall: '#ECE6D8', roof: '#5C626A', shape: 'gable', parts: [part([10, 20, 18, 30], 3, 2)], pad: 4 }, slope);
    expect([cut.floor, cut.top]).toEqual([4, 7]);
    expect(cut.base).toBeCloseTo(4 - 0.4, 6);
  });

  test('with a look, drawn from it: as tall as its real storeys, lit by its hours', () => {
    const h = { id: 2, chunk: 0, box: [42.93, 4.93, 53.07, 14.07] as [number, number, number, number], storeys: 1, wall: '#ffffff', roof: '#000000', shape: 'gable' as const, parts: [part([42.93, 4.93, 53.07, 14.07], 3, 2)], pad: 0 };
    const [plain] = lotBuilding(h, flat);
    expect(plain.top).toBeCloseTo(3, 6);
    // (A church: its tower beside it, its spire's top over its roof's.)
    const [t, tower] = lotBuilding({ ...h, storeys: 2, look: { use: 'church', storeys: 2, roof: 'gable', look: 0 }, hours: { open: 540, close: 1080, days: 127 } }, flat);
    expect(t.top).toBeCloseTo(6, 6);
    expect([t.roof, t.style]).toEqual(['gable', STYLE.church[0]]);
    expect(tower.roof).toBe('spire');
    expect(tower.top).toBeGreaterThan(t.top + t.rise);
    expect(litAt(t, 600, 0)).toBe(t.lit);
    expect(litAt(t, 1200, 0)).toBeLessThan(t.lit);
    // (The church's own stone and slate.)
    expect(t.wall).not.toEqual(plain.wall);
    // A flats building: a home's lights, by the hour; a shop with flats over it, its flats' while it is closed.
    const [flats] = lotBuilding({ ...h, storeys: 2, look: { use: 'flats', storeys: 2, roof: 'flat', look: 1 } }, flat);
    expect([flats.home, flats.style]).toEqual([true, STYLE.flats[0]]);
    const [mixed] = lotBuilding({ ...h, storeys: 2, look: { use: 'shop', storeys: 2, roof: 'flat', look: 1 }, hours: { open: 540, close: 1080, days: 127 }, flats: true }, flat);
    expect(mixed.flats).toBe(true);
    expect(litAt(mixed, 20 * 60, 0)).toBeCloseTo(STYLE.flats[1]);
    expect(litAt(mixed, 3 * 60, 0)).toBeLessThan(STYLE.flats[1] * 0.2);
    // (One storey: no flats over it.)
    expect(lotBuilding({ ...h, look: { use: 'shop', storeys: 1, roof: 'flat', look: 1 }, flats: true }, flat)[0].flats).toBeUndefined();
  });
});

describe('lights', () => {
  test('go by the life there: a shop’s while it is open, a home’s by the hour, a lot’s house’s while someone is up', () => {
    // Homes: all their lights in the evening, most out by half past midnight, on again at breakfast.
    expect(homeLights(20 * 60)).toBe(1);
    expect(homeLights(23 * 60)).toBeLessThan(0.8);
    expect(homeLights(24 * 60 + 30)).toBeCloseTo(0.15);
    expect(homeLights(3 * 60)).toBeCloseTo(0.15);
    expect(homeLights(8 * 60)).toBeCloseTo(0.7);
    let last = 1;
    for (let m = 22 * 60; m <= 24 * 60 + 30; m += 10) {
      expect(homeLights(m)).toBeLessThanOrEqual(last);
      last = homeLights(m);
    }
    // Open 09:00 to 18:00, Monday to Saturday.
    const hours = { open: 540, close: 1080, days: 0b0111111 };
    const s = lotLookBuilding([0, 0, 10, 6], look('shop'), 2, 0, flat, hours);
    expect(s.hours).toEqual(hours);
    expect(litAt(s, 17 * 60, 0)).toBe(s.lit);
    expect(litAt(s, 20 * 60, 0)).toBeLessThan(s.lit * 0.1);
    expect(litAt(s, 12 * 60, 6)).toBeLessThan(s.lit * 0.1);
    // A pub open 17:00 to 01:00 on Friday is still open after midnight on Saturday's clock.
    const pub = { lit: 0.9, hours: { open: 1020, close: 60, days: 0b0010000 } };
    expect(litAt(pub, 23 * 60, 4)).toBe(0.9);
    expect(litAt(pub, 30, 5)).toBe(0.9);
    expect(litAt(pub, 120, 5)).toBeLessThan(0.1);
    expect(litAt(pub, 23 * 60, 5)).toBeLessThan(0.1);
    // A flats building: by the hour. A place without hours: its own share.
    const home = lotLookBuilding([0, 0, 10, 6], look('flats'), 2, 0, flat);
    expect(home.home).toBe(true);
    expect(litAt(home, 20 * 60, 0)).toBe(home.lit);
    expect(litAt(home, 2 * 60, 0)).toBeCloseTo(home.lit * 0.15);
    const hall = lotLookBuilding([0, 0, 10, 6], look('hall'), 2, 0, flat);
    expect(litAt(hall, 2 * 60, 0)).toBe(hall.lit);
    // A lot's house far off: lit only while someone is up in it.
    const lot = { id: 3, chunk: 0, box: [10, 20, 18, 30] as const, storeys: 2, wall: '#ECE6D8', roof: '#5C626A', shape: 'hip' as const, parts: [part([10, 20, 18, 30], 6, 2.4)], pad: 0 };
    expect(lotBuilding(lot, flat, true)[0].lit).toBeGreaterThan(0.3);
    expect(lotBuilding(lot, flat, false)[0].lit).toBe(0);
  });
});

describe('the unit meshes', () => {
  test("every unit mesh's faces face out from its middle (none drawn inside out)", () => {
    const meshes = unitMeshes();
    for (const key of ['walls', ...ROOFS] as const) {
      const d = meshes[key].meshData();
      const p = d.positions;
      for (let i = 0; i < d.indices.length; i += 3) {
        const [a, b, c] = [d.indices[i], d.indices[i + 1], d.indices[i + 2]];
        const e1 = [p[a * 3] - p[b * 3], p[a * 3 + 1] - p[b * 3 + 1], p[a * 3 + 2] - p[b * 3 + 2]];
        const e2 = [p[c * 3] - p[b * 3], p[c * 3 + 1] - p[b * 3 + 1], p[c * 3 + 2] - p[b * 3 + 2]];
        const n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
        // (The winding's normal agrees with the vertex normal: `Geo`'s convention.)
        const vn = [d.normals[a * 3], d.normals[a * 3 + 1], d.normals[a * 3 + 2]];
        expect(n[0] * vn[0] + n[1] * vn[1] + n[2] * vn[2]).toBeGreaterThanOrEqual(0);
      }
      expect(d.indices.length / 3).toBeLessThan(80);
    }
  });
});
