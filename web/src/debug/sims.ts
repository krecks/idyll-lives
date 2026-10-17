/**
 * `?debug`'s `__sims()`: the residents as the newest snapshot shows them, for browser checks that
 * wait for "the resident is at the object, sitting" instead of timing their screenshots.
 */

import type { Content } from '../content/content';
import type { SimView } from '../core/protocol';
import type { SnapshotLayout } from '../core/snapshot';

const POSES = ['stand', 'sit', 'lie'] as const;

export interface LiveSim {
  id: number;
  name: string;
  household: number;
  /** Plot they are on (null on the street or away). */
  plot: number | null;
  /** Away from the lots (at work, at school). */
  away: boolean;
  /** Lot tiles (`x`, `z`) and storey (0 the ground; fractions on the stairs). */
  x: number;
  z: number;
  storey: number;
  /** Facing, radians. */
  yaw: number;
  pose: 'stand' | 'sit' | 'lie';
  moving: boolean;
  /** The object they use or walk to use (`__world().objects` id), its content id, and the interaction's id. */
  object: number | null;
  def: string | null;
  interaction: string | null;
  /** They are at the object and using it (not walking to it): `action` is the animation it plays. */
  using: boolean;
  action: string | null;
  /** What they carry (content item id), and the recipe they cook. */
  held: string | null;
  recipe: string | null;
  /** Talking with (another resident's id). */
  partner: number | null;
  /** Needs by content id, 0..1 (only for residents with details: the player's household, the inspected). */
  needs: Record<string, number> | null;
  /** What they do now and the queue after it, as the resident's panel lists it. */
  doing: string | null;
  queue: string[];
}

/** Every resident in the snapshot `snap`. */
export function liveSims(snap: Float32Array, layout: SnapshotLayout, content: Content, views: readonly SimView[]): LiveSim[] {
  const k = layout.sim;
  const field = (o: number, at: number | undefined, none = -1) => (at === undefined ? none : snap[o + at]);
  const out: LiveSim[] = [];
  for (let i = 0; i < snap[layout.header.simCount]; i++) {
    const o = layout.headerLen + i * layout.simStride;
    const id = snap[o + k.id];
    const view = views.find((v) => v.id === id);
    const objectId = snap[o + k.object] | 0;
    const def = content.objectList[field(o, k.objectDef) | 0];
    const interaction = field(o, k.interaction) | 0;
    const action = snap[o + k.action] | 0;
    const held = field(o, k.held, 0) | 0;
    const recipe = field(o, k.recipe, 0) | 0;
    const partner = snap[o + k.partner] | 0;
    out.push({
      id,
      name: view?.name ?? '',
      household: view?.household ?? -1,
      plot: view?.plot ?? null,
      away: snap[o + k.away] > 0,
      x: snap[o + k.x],
      z: snap[o + k.z],
      storey: field(o, k.height, 0),
      yaw: snap[o + k.yaw],
      pose: POSES[snap[o + k.pose] | 0] ?? 'stand',
      moving: snap[o + k.moving] > 0,
      object: objectId >= 0 ? objectId : null,
      def: objectId >= 0 && def ? def.id : null,
      interaction: objectId >= 0 && def ? (def.interactions[interaction]?.id ?? null) : null,
      using: objectId >= 0 && action >= 0,
      action: action >= 0 ? (layout.actions[action] ?? null) : null,
      held: held > 0 ? (layout.items?.[held - 1] ?? null) : null,
      recipe: recipe > 0 ? (layout.recipes?.[recipe - 1] ?? null) : null,
      partner: partner >= 0 ? partner : null,
      needs: view?.detail ? Object.fromEntries(content.needs.map((n, j) => [n.id, view.needs[j] ?? 0])) : null,
      doing: view?.actions[0]?.label ?? null,
      queue: view?.actions.slice(1).map((a) => a.label) ?? [],
    });
  }
  return out;
}
