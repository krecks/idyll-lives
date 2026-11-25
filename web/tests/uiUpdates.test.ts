// `game/uiUpdates.ts`: the HUD's copy of the worker's view, from a full update and the changes after it.
import { expect, test } from 'bun:test';
import type { UiUpdate } from '../src/core/protocol';
import { UiMirror } from '../src/game/uiUpdates';

const update = (u: Partial<UiUpdate>): UiUpdate => ({ full: false, day: 1, weekday: 0, minute: 0, speed: 1, quiet: false, autonomy: true, lifespan: 'normal' as UiUpdate['lifespan'], playerMoves: false, ...u });

test('a full update, then changes on top: a null field and lost details are dropped, a resident gone is gone, households keep their money', () => {
  const mirror = new UiMirror();
  expect(mirror.apply(update({ sims: [{ id: 1, name: 'Ann' }] }))).toBeNull();
  mirror.apply(
    update({
      full: true,
      sims: [
        { id: 1, name: 'Ann' },
        { id: 2, name: 'Bo', detail: true, pension: 40, grade: 3, mood: 5, needs: [{ id: 'hunger', value: 0.5 }] as never },
        { id: 3, name: 'Cy' },
      ],
      households: [{ id: 7, funds: 1200, rent: null, bills: null }],
    }),
  );
  const kept = mirror.apply(update({ sims: [{ id: 2, pension: null, detail: false }] }))!;
  expect(kept.sims!.map((s) => s.id)).toEqual([1, 2, 3]);
  const bo = kept.sims![1];
  expect(bo.name).toBe('Bo');
  expect(bo.detail).toBe(false);
  expect('pension' in bo || 'grade' in bo).toBe(false);
  expect(bo.needs).toEqual([]);
  expect(bo.mood).toBe(0);
  expect(kept.relationships).toBeNull();
  // (Someone new in an old slot: still listed in id order, then and after.)
  expect(mirror.apply(update({ simsGone: [1], sims: [{ id: 1, name: 'Di' }] }))!.sims!.map((s) => s.name)).toEqual(['Di', 'Bo', 'Cy']);
  const after = mirror.apply(update({ simsGone: [3], sims: [{ id: 2, mood: 1 }] }))!;
  expect(after.sims!.map((s) => s.name)).toEqual(['Di', 'Bo']);
  expect(mirror.households.get(7)?.funds).toBe(1200);
});
