// What every content object plays for every action tag (`render/shared/characters/actions.ts` `ACTIONS`, in the
// manner of its content `how`, `actionOf`): the standing clips, the seated clip and upper layer, the hands and the
// prop in hand, for a child, a teen and a grown-up (1.12, 1.47, 1.77 m). Each tag's row for no object, then the
// objects (base, catalogs and packs) whose rows differ from it. A change meant to change how residents use things: `bun test --update-snapshots`.
import { describe, expect, test } from 'bun:test';
import { loadContent, usePublicFetch } from './support/content';

usePublicFetch();

describe('action clips', async () => {
  const content = await loadContent();
  const { ACTIONS, MANNERS, PROPS, actionOf } = await import('../src/render/shared/characters/actions');
  const propName = new Map<unknown, string>(Object.entries(PROPS).map(([name, p]) => [p, name]));
  const BANDS = [
    ['child', 1.12],
    ['teen', 1.47],
    ['adult', 1.77],
  ] as const;

  const row = (tag: string, def: string, height: number): string => {
    const a = actionOf(tag, content.object(def)?.how?.[tag])!;
    const stand = typeof a.stand === 'function' ? a.stand(def, height) : a.stand;
    const { sit, upper, hands } = a;
    const prop = typeof a.prop === 'function' ? a.prop(def, '', false, false) : a.prop;
    return `stand=${stand?.join(',') ?? '-'} sit=${sit ?? '-'} upper=${upper ?? '-'} hands=${hands ?? '-'} prop=${prop ? (propName.get(prop) ?? prop.key) : '-'}`;
  };
  const rows = (tag: string, def: string): string[] => {
    const all = BANDS.map(([band, h]) => [band, row(tag, def, h)] as const);
    return all.every(([, r]) => r === all[0][1]) ? [all[0][1]] : all.map(([band, r]) => `@${band} ${r}`);
  };

  test('per tag, per object, per height', () => {
    const defs = [...new Set(content.objectList.map((d) => d.id))].sort();
    const lines: string[] = [];
    for (const tag of Object.keys(ACTIONS).sort()) {
      const base = rows(tag, '');
      lines.push(...base.map((r) => `${tag} | * | ${r}`));
      for (const def of defs) {
        const own = rows(tag, def);
        if (own.join('\n') !== base.join('\n')) lines.push(...own.map((r) => `${tag} | ${def} | ${r}`));
      }
    }
    expect(lines.join('\n')).toMatchSnapshot();
  });

  test("every object's `how` is a manner for an action tag", () => {
    const wrong = content.objectList.flatMap((d) => Object.entries(d.how ?? {}).filter(([tag, how]) => !ACTIONS[tag] || !MANNERS[how]).map(([tag, how]) => `${d.id}: ${tag} ${how}`));
    expect(wrong).toEqual([]);
  });
});
