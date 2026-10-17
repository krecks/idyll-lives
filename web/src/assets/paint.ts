import type { ModelEntry, PaintSlot } from './types';

/** The parts of a model the player may colour (its `paint`), in order: the main part first. */
export function paintSlots(entry: ModelEntry | undefined): PaintSlot[] {
  return (entry?.paint ?? []).map((p) => (typeof p === 'string' ? { material: p, label: p.charAt(0).toUpperCase() + p.slice(1) } : p));
}
