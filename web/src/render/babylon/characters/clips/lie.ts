/**
 * Lying down (phase 2, stream C): a `lie` clip on the back in the standing convention (the runtime
 * still turns the Sim flat), for `CLIP_OF.lie` and `lyingPose`. Until it exists the runtime lies on
 * the `idle` clip with straightened legs.
 */

import type { ClipDef } from './dsl';

export function lieClips(): ClipDef[] {
  return [];
}
