/**
 * What the town's servers wear (`game/staff.ts`): their post's uniform (content `staff.uniform`), a shop's apron
 * without one, every garment one the body of their gender has.
 */
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import type { Content, ObjectDef } from '../src/content/content';
import type { ObjectPlacement } from '../src/core/protocol';
import { staffPosts, uniformFor, UNIFORMS } from '../src/game/staff';
import { GARMENTS, WORKWEAR } from '../src/render/shared/characters/outfit';

const civic = JSON.parse(readFileSync(new URL('../public/content/civic.json', import.meta.url), 'utf8')) as { objects: ObjectDef[] };
const def = (id: string) => civic.objects.find((o) => o.id === id);

describe("servers' uniforms", () => {
  test('a post without a uniform: the apron; a teacher, a receptionist, a desk officer: their own', () => {
    expect(uniformFor(undefined, 'male').top).toBe('top.apron');
    expect(uniformFor({ staff: { at: [0, 0], hours: [8, 18] } } as ObjectDef, 'female').top).toBe('top.apron');
    expect(uniformFor(def('teacherDesk'), 'female').top).toBe('top.jacket');
    expect(uniformFor(def('receptionDesk'), 'female').top).toBe('top.blouse');
    expect(uniformFor(def('receptionDesk'), 'male').top).toBe('top.shirt');
    expect(uniformFor(def('dutyDesk'), 'male').body).toBe(UNIFORMS.service.body);
    expect(uniformFor(def('hotelReception'), 'female')).toMatchObject({ top: 'top.jacket', bottom: 'bottom.skirt' });
    expect(uniformFor(def('hotelReception'), 'male')).toMatchObject({ top: 'top.suit', bottom: 'bottom.suit' });
  });

  test("every content post's uniform is one there is, and every one fits both bodies", () => {
    for (const o of civic.objects) if (o.staff?.uniform) expect(Object.keys(UNIFORMS)).toContain(o.staff.uniform);
    for (const name of Object.keys(UNIFORMS)) {
      for (const g of ['female', 'male'] as const) {
        const u = uniformFor({ staff: { at: [0, 0], hours: [0, 24], uniform: name } } as ObjectDef, g);
        expect([...GARMENTS.TOPS[g], ...WORKWEAR]).toContain(u.top);
        expect(GARMENTS.BOTTOMS[g]).toContain(u.bottom);
        expect(GARMENTS.SHOES).toContain(u.shoes);
      }
    }
  });

  test("a server dresses for the first of their posts with a uniform, found by the thing's stable id", () => {
    const content = { object: (id: string) => def(id) ?? ({ id, staff: { at: [0, 0], hours: [8, 18] } } as ObjectDef) } as Pick<Content, 'object'>;
    const thing = (id: number, kind: string) => ({ id, def: kind }) as ObjectPlacement;
    const world = {
      objects: [thing(10, 'shopCounter'), thing(11, 'barCounter'), thing(12, 'teacherDesk')],
      staff: [
        { seed: 1, posts: [10, 11] },
        { seed: 2, posts: [12] },
        { seed: 3, posts: [10] },
        { seed: 4, posts: [99] },
      ],
    };
    expect(staffPosts(content, world).map((d) => d?.id)).toEqual(['barCounter', 'teacherDesk', 'shopCounter', undefined]);
  });
});
