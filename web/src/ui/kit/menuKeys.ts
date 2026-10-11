/**
 * Arrow keys for a menu's list of buttons: ↑/↓ move between the enabled entries (wrapping round),
 * Home/End go to the first and last. Put it on the list's `onkeydown`.
 */
export function menuArrows(e: KeyboardEvent): void {
  const list = e.currentTarget as HTMLElement;
  const items = [...list.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')];
  if (!items.length) return;
  const at = items.indexOf(document.activeElement as HTMLButtonElement);
  let next: number;
  if (e.key === 'ArrowDown') next = at < 0 ? 0 : (at + 1) % items.length;
  else if (e.key === 'ArrowUp') next = at < 0 ? items.length - 1 : (at - 1 + items.length) % items.length;
  else if (e.key === 'Home') next = 0;
  else if (e.key === 'End') next = items.length - 1;
  else return;
  e.preventDefault();
  items[next].focus();
}

/** Before anything has focus, ↑/↓ start in the menu's list (from the window's `onkeydown`). */
export function enterMenu(e: KeyboardEvent, list: HTMLElement | undefined): void {
  if (!list || (e.key !== 'ArrowDown' && e.key !== 'ArrowUp')) return;
  if (document.activeElement && document.activeElement !== document.body) return;
  const first = list.querySelector<HTMLButtonElement>('button:not(:disabled)');
  if (!first) return;
  e.preventDefault();
  first.focus();
}
