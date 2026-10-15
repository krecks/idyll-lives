/**
 * The browser's own right-click menu (Back, Reload, Inspect…) never opens over the game: not on
 * the 3D view, not on a menu, panel, card or toast. Only text fields keep it, so paste and
 * spellcheck still work there (the buy menu's search, a sim's name).
 *
 * Only the default is prevented; the event still reaches every listener, so right-button game
 * controls are unaffected (right-drag pans the camera, a right-click cancels a wall being drawn:
 * `game/input.ts`). The debug pages (gallery, clips, poses, characters) don't load this and keep
 * the native menu, for Inspect.
 */

/** Input types the browser edits as text (its menu offers paste and spelling there). */
const TEXT_INPUTS = new Set(['text', 'search', 'email', 'url', 'tel', 'password', 'number']);

function editable(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable || target instanceof HTMLTextAreaElement) return true;
  return target instanceof HTMLInputElement && TEXT_INPUTS.has(target.type);
}

/** Capture phase on the window: before any component can stop the event. */
export function suppressContextMenu(): void {
  window.addEventListener(
    'contextmenu',
    (e) => {
      if (!editable(e.target)) e.preventDefault();
    },
    true,
  );
}
