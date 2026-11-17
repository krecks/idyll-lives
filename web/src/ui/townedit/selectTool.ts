/**
 * The Select tool: a click on a lot, a street, a backdrop or civic building, a village or the main
 * town picks it, and its card (`SelectCard.svelte`) says what can be done to it. What the pointer
 * is over is lit as it goes.
 */

import SelectCard from './SelectCard.svelte';
import type { TownEditor, TownTool } from './tools';

export const selectTool: TownTool = {
  id: 'select',
  label: 'Select',
  icon: 'icon.ui.search',
  does: 'Click a lot, a street, a building or a village to see what can be done to it.',
  stage: (editor: TownEditor) => ({
    down(x, y) {
      editor.select(editor.pick(x, y));
      return true;
    },
    move: () => {},
    up: () => {},
    hover(x, y) {
      editor.hover(editor.pick(x, y));
    },
    leave: () => editor.hover(null),
    cancel: () => {},
  }),
  card: SelectCard,
};
