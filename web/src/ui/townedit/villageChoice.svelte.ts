/**
 * What the Village tool does, as its card (`VillageCard.svelte`) and the village card's "Move"
 * (`SelectCard.svelte`) set it: a new village laid along or turned across, or a village being moved
 * (its area: a click puts it where the pointer is). Kept while the tool is put down and taken up
 * again, but for the village being moved, which is let go with the tool.
 */

import type { Edited } from '../../game/edit/edited';
import { toast } from '../state.svelte';
import type { TownEditor } from './tools';
import { sentence } from './words';

export const villageChoice = $state({
  /** A new village's shape turned across (its box `d` × `w`, as the generator turns one). */
  swap: false,
  /** The village being moved (its area), or none: the tool places a new one. */
  moving: null as number | null,
});

/** The Village tool taken in hand to move village `area` (the card's "Move"): a click puts it where the pointer is. */
export function moveWithTool(editor: TownEditor, area: number): void {
  if (editor.toolId !== 'village') editor.take('village');
  villageChoice.moving = area;
}

/** A village edit made (`editor.apply`): what it left for the player to know (no road to it, an old road left ending in the country) said as toasts. Whether it was made. */
export function applyVillage(editor: TownEditor, edited: Edited): boolean {
  if (!editor.apply(edited)) return false;
  if ('draft' in edited) for (const note of edited.notes ?? []) toast(sentence(note), 6000);
  return true;
}
