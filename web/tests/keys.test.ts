/**
 * The game's key routing (`game/input.ts` `routeKey`): what reaches the game's shortcuts with a
 * dialog or the pause menu open, with Ctrl, ⌘ or Alt held, and where Tab goes.
 */
import { describe, expect, test } from 'bun:test';
import { routeKey, type KeyContext, type KeyPress } from '../src/game/input';

const key = (k: string, mods: Partial<Omit<KeyPress, 'key'>> = {}): KeyPress => ({ key: k, ctrlKey: false, metaKey: false, altKey: false, shiftKey: false, ...mods });
const scene: KeyContext = { paused: false, dialog: false, onScene: true };

describe('key routing', () => {
  test('on the scene, the game takes its keys', () => {
    for (const k of [' ', '1', 'b', 'v', 'c', 'o', 'p', 'f', 'Tab', 'Delete', 'PageUp']) expect(routeKey(key(k), scene)).toBe('game');
    expect(routeKey(key('R', { shiftKey: true }), scene)).toBe('game');
    expect(routeKey(key('Escape'), scene)).toBe('escape');
  });

  test('behind a dialog (the planner, the job board, settings) the world rests', () => {
    const dialog = { ...scene, dialog: true };
    for (const k of [' ', '3', 'Tab', 'b', 'v', 'c', 'o', 'Backspace']) expect(routeKey(key(k), dialog)).toBe('none');
    // Its own Escape closes it (`kit/Modal.svelte`, `JobBoard.svelte`): the game's must not act behind it.
    expect(routeKey(key('Escape'), dialog)).toBe('none');
    expect(routeKey(key('z', { ctrlKey: true }), dialog)).toBe('none');
    // (The debug keys work everywhere.)
    expect(routeKey(key('F8'), dialog)).toBe('game');
  });

  test('under the pause menu only Escape (it closes the menu) and the debug keys', () => {
    const paused = { ...scene, paused: true };
    expect(routeKey(key('Escape'), paused)).toBe('escape');
    for (const k of [' ', '1', 'b', 'Tab']) expect(routeKey(key(k), paused)).toBe('none');
    expect(routeKey(key('F3'), paused)).toBe('game');
  });

  test('with Ctrl, ⌘ or Alt held a key is the browser’s, but for undo and redo', () => {
    expect(routeKey(key('v', { metaKey: true }), scene)).toBe('none');
    expect(routeKey(key('p', { ctrlKey: true }), scene)).toBe('none');
    expect(routeKey(key('f', { ctrlKey: true }), scene)).toBe('none');
    expect(routeKey(key('c', { ctrlKey: true }), scene)).toBe('none');
    expect(routeKey(key('Backspace', { metaKey: true }), scene)).toBe('none');
    expect(routeKey(key('b', { altKey: true }), scene)).toBe('none');
    expect(routeKey(key('z', { ctrlKey: true }), scene)).toBe('undo');
    expect(routeKey(key('z', { metaKey: true }), scene)).toBe('undo');
    expect(routeKey(key('Z', { metaKey: true, shiftKey: true }), scene)).toBe('redo');
    expect(routeKey(key('y', { ctrlKey: true }), scene)).toBe('redo');
    expect(routeKey(key('y', { metaKey: true }), scene)).toBe('none');
  });

  test('Tab picks the next resident only from the page or the canvas', () => {
    expect(routeKey(key('Tab'), scene)).toBe('game');
    expect(routeKey(key('Tab', { shiftKey: true }), scene)).toBe('game');
    expect(routeKey(key('Tab'), { ...scene, onScene: false })).toBe('none');
    // (Other keys work wherever the focus is.)
    expect(routeKey(key('m'), { ...scene, onScene: false })).toBe('game');
  });
});
