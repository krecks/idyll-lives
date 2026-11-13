/**
 * The town view's HUD state (docs/design/living-town.md, Town tags): whether the town view is in far enough for
 * the tags (`townView.blend` from 0.2; the household's own from its start), the tag filters (kept in this browser), and the tag the player clicked.
 */

import type { TagKind } from './overlap';

/** A tag the player clicked, kept open as a card: one of your people (visit or call), or a place. */
export type Chosen = ({ kind: 'person'; sim: number; plot: number | null } | { kind: 'place'; place: number; plot: number | null }) & {
  /** The tag's words when it was clicked. */
  title: string;
  detail: string;
};

const KEY = 'idyll.townTags';
const ALL: Record<TagKind, boolean> = { household: true, people: true, places: true, sale: true, names: true };

function load(): Record<TagKind, boolean> {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<Record<TagKind, boolean>> | null;
    return { ...ALL, ...(saved ?? {}) };
  } catch {
    return { ...ALL };
  }
}

class TownHud {
  /** The town view is in (the tags show): `townView.blend` at `TAGS_FROM` or more. */
  on = $state(false);
  /** The tags' fade (0..1), rounded to steps of 0.05. */
  fade = $state(0);
  /** The town view has begun (`townView.blend` over 0, far enough for the household's own tags to be seen: `MINE_SEEN`): they show from there, by `mine` (0..1, in steps of 0.05). */
  near = $state(false);
  mine = $state(0);
  filters = $state<Record<TagKind, boolean>>(load());
  chosen = $state.raw<Chosen | null>(null);
  /** The household's residents away at no place the town has (at work, "at work"): no tag, so the Today card lists them. */
  away = $state.raw<{ id: number; name: string; text: string }[]>([]);

  toggle(kind: TagKind): void {
    this.filters[kind] = !this.filters[kind];
    try {
      localStorage.setItem(KEY, JSON.stringify(this.filters));
    } catch {
      // (Storage full or off: the filter holds for this session.)
    }
  }
}

export const townHud = new TownHud();
