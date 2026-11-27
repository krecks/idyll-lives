// What a resident's action sounds like (`audio/world.ts` `soundOf`): the defaults by animation, and eating by what's
// on the plate (cutlery only for a plate eaten with a fork), with the content's own `sound` or `sounds` first.
import { describe, expect, mock, test } from 'bun:test';
import type { Content } from '../src/content/content';

// (The game's state and the settings are Svelte's, the mixer the browser's: not needed to say what a use sounds like.)
mock.module('../src/ui/state.svelte', () => ({ game: {} }));
mock.module('../src/settings/settings.svelte', () => ({ settings: {} }));
mock.module('../src/audio/mixer', () => ({ audioContext: () => null, bus: () => null, musicGain: () => 1, space: () => null, updateVolumes: () => {} }));
const { knocks, shutSoundOf, soundOf } = await import('../src/audio/world');

/** Content with just the objects `soundOf` reads. */
const content = {
  objectList: [
    { id: 'diningChair', interactions: [{ id: 'sit', anim: 'sit' }] },
    { id: 'quietTable', interactions: [{ id: 'dine', anim: 'eat', sound: 'none' }] },
    { id: 'hushedTable', sounds: { eat: 'none' }, interactions: [{ id: 'dine', anim: 'eat' }] },
    { id: 'computerDesk', interactions: [{ id: 'code', anim: 'type' }] },
    { id: 'plant', interactions: [{ id: 'water', anim: 'water' }] },
    { id: 'hedge', interactions: [{ id: 'trim', anim: 'prune' }] },
    { id: 'doorbell', sounds: { knock: 'doorbell' }, interactions: [{ id: 'ring', anim: 'knock' }] },
    { id: 'curtains', interactions: [] },
    { id: 'blinds', sounds: { shut: 'slats' }, interactions: [] },
    { id: 'awning', sounds: { shut: 'none' }, interactions: [] },
  ],
  recipes: [],
} as unknown as Content;
const def = (id: string) => content.objectList.findIndex((o) => o.id === id);

describe('soundOf', () => {
  test('a plate eaten with a fork sounds of cutlery, at a seat or with no use', () => {
    expect(soundOf(content, 'eat', def('diningChair'), 0, 0, 'mealPlate')).toBe('cutlery');
    expect(soundOf(content, 'eat', def('diningChair'), -1, 0, 'cakeSlice')).toBe('cutlery');
    expect(soundOf(content, 'eat', -1, -1, 0, 'leftovers')).toBe('cutlery');
  });

  test('what is eaten another way, or nothing on a plate, is silent', () => {
    for (const plate of ['sandwich', 'iceCream', 'iceCreamCone', 'noodles', 'apple', '']) {
      expect(soundOf(content, 'eat', def('diningChair'), 0, 0, plate)).toBeUndefined();
    }
    expect(soundOf(content, 'eat', def('diningChair'), 0)).toBeUndefined();
  });

  test("a use's or an object's `none` wins over a fork", () => {
    expect(soundOf(content, 'eat', def('quietTable'), 0, 0, 'mealPlate')).toBeUndefined();
    expect(soundOf(content, 'eat', def('hushedTable'), 0, 0, 'mealPlate')).toBeUndefined();
  });

  test('a plate on another animation is ignored', () => {
    expect(soundOf(content, 'sit', def('diningChair'), 0, 0, 'mealPlate')).toBeUndefined();
    expect(soundOf(content, 'watch', def('diningChair'), 0, 0, 'mealPlate')).toBe('tv');
  });

  test('typing, watering and pruning by their animations', () => {
    expect(soundOf(content, 'type', def('computerDesk'), 0)).toBe('typing');
    expect(soundOf(content, 'water', def('plant'), 0)).toBe('wateringCan');
    expect(soundOf(content, 'prune', def('hedge'), 0)).toBe('shears');
  });
});

describe('knocks and curtains', () => {
  test('a visitor at the door knocks; a doorbell rung chimes instead', () => {
    expect(knocks(content, 'knock', -1, -1)).toBe(true);
    expect(knocks(content, 'knock', def('doorbell'), 0)).toBe(false);
    expect(soundOf(content, 'knock', def('doorbell'), 0)).toBe('doorbell');
    expect(knocks(content, 'sit', def('diningChair'), 0)).toBe(false);
  });

  test('drawn curtains swish, blinds clatter, an awning is silent', () => {
    expect(shutSoundOf(content, def('curtains'))).toBe('curtain');
    expect(shutSoundOf(content, -1)).toBe('curtain');
    expect(shutSoundOf(content, def('blinds'))).toBe('slats');
    expect(shutSoundOf(content, def('awning'))).toBeUndefined();
  });
});
