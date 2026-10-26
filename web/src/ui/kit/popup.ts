/**
 * A small popup over the world (an object's pie menu, the social menu, the town map) has no
 * backdrop: the world behind it keeps every pointer gesture, so the camera still turns, pans and
 * zooms with the popup open, and a plain click on the world closes it and does what a click there
 * always does (`click` in `game/session.ts`). A press anywhere else outside the popup (the HUD, a
 * panel) closes it too, but on the button that opens and closes it (`aria-controls` naming the
 * popup's `id`), which does so itself. Put it on the popup's element: `use:closeOutside={close}`.
 */
import type { Action } from 'svelte/action';

export const closeOutside: Action<HTMLElement, () => void> = (node, close) => {
  let done = close;
  const onDown = (e: PointerEvent) => {
    const t = e.target;
    if (!(t instanceof Element) || node.contains(t)) return;
    if (node.id && t.closest(`[aria-controls="${node.id}"]`)) return;
    // The world (its canvas and the labels over it) closes the popup on a click, not a drag.
    if (t.closest('.game-host')) return;
    done();
  };
  window.addEventListener('pointerdown', onDown, true);
  return {
    update(next: () => void) {
      done = next;
    },
    destroy() {
      window.removeEventListener('pointerdown', onDown, true);
    },
  };
};
