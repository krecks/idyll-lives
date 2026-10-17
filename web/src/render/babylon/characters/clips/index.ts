/**
 * The residents' animation clips, made in code (`dsl.ts`) and baked when the character set loads.
 * Every name the runtime plays (characters.ts `ACTIONS`, `choose`, reactions) must be here; see
 * docs/characters-format.md for the list and the conventions. Phase-2 clips are here too, ready for
 * the runtime: `lie`, `chop`, `stir`, `flip`, `drink`, `carry`, `carryOne` and `wave`. A baby in
 * the crib has its own lying clips (`baby.ts`); the drinking fountain and the climbing frame theirs, in
 * three sizes (`street.ts`). Arm and leg placement uses the IK helpers in
 * `ik.ts` (on the reference skeleton, while the clips bake).
 *
 * In dev, editing a clip file re-bakes the clips of every loaded set in place (Vite HMR), so the
 * pose bench updates without a reload.
 */

import type { CharacterSet, Clip } from '../data';
import { actionClips } from './action';
import { babyClips } from './baby';
import { bake, type ClipDef } from './dsl';
import { gardenClips } from './garden';
import { idleClips } from './idle';
import { lieClips } from './lie';
import { locomotionClips } from './locomotion';
import { seatedClips } from './seated';
import { streetClips } from './street';
import { workClips } from './work';

/** Every clip definition. */
export function clipDefs(): ClipDef[] {
  return [...idleClips(), ...locomotionClips(), ...seatedClips(), ...workClips(), ...gardenClips(), ...actionClips(), ...lieClips(), ...babyClips(), ...streetClips()];
}

const live = new Set<CharacterSet>();

/** The set's clips, baked from the definitions. */
export function buildClips(set: CharacterSet): Map<string, Clip> {
  if (import.meta.hot) live.add(set);
  return bake(clipDefs(), set);
}

const hot = import.meta.hot;
if (hot) {
  // A new version of this module (any clip file changed): re-bake the clips of the sets the old one built.
  const sets = (hot.data.sets as Set<CharacterSet> | undefined) ?? live;
  for (const set of sets) {
    live.add(set);
    for (const [name, clip] of bake(clipDefs(), set)) set.clips.set(name, clip);
  }
  hot.dispose((data) => (data.sets = live));
  hot.accept();
}
