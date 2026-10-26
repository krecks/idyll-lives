/**
 * The residents' animation clips, made in code (`dsl.ts`) and baked when the character set loads.
 * Every name the runtime plays (characters.ts `ACTIONS`, `choose`, reactions) must be here; see
 * docs/characters-format.md for the list and the conventions. Phase-2 clips are here too, ready for
 * the runtime: `lie`, `chop`, `stir`, `flip`, `drink`, `carry`, `carryOne` and `wave`. A baby in
 * the crib has its own lying clips (`baby.ts`); children's play inside the ball pit and the play tent theirs (`play.ts`); the drinking fountain and the climbing frame theirs, in
 * three sizes (`street.ts`); games and sport at things theirs (`games.ts`), some in sizes too, and so are the reaches
 * up to things at fixed heights (`reach.ts`), the hoop shot (`shot.ts`) and the hands on low tops (`low.ts`); the slide's round,
 * the bunk's ladder and the rides theirs (`rides.ts`), in sizes; the socials' gestures and reactions and the phone held in front
 * theirs (`social.ts`); brushing the hair and a curled side lie are in `self.ts`; buying from a vendor and serving in `serve.ts`. Arm and leg placement uses the IK helpers in
 * `ik.ts` (on the reference skeleton, while the clips bake).
 *
 * In dev, editing a clip file re-bakes the clips of every loaded set in place (Vite HMR), so the
 * pose bench updates without a reload.
 */

import type { CharacterSet, Clip } from '../data';
import { actionClips } from './action';
import { babyClips } from './baby';
import { bake, type ClipDef } from './dsl';
import { gameClips } from './games';
import { gardenClips } from './garden';
import { idleClips } from './idle';
import { lieClips } from './lie';
import { rideClips } from './rides';
import { locomotionClips } from './locomotion';
import { lowClips } from './low';
import { reachClips } from './reach';
import { seatedClips } from './seated';
import { socialClips } from './social';
import { playClips } from './play';
import { selfClips } from './self';
import { serveClips } from './serve';
import { shotClips } from './shot';
import { streetClips } from './street';
import { useClips } from './uses';
import { workClips } from './work';

/** Every clip definition. */
export function clipDefs(): ClipDef[] {
  return [...idleClips(), ...locomotionClips(), ...seatedClips(), ...workClips(), ...gardenClips(), ...actionClips(), ...lieClips(), ...babyClips(), ...streetClips(), ...gameClips(), ...useClips(), ...socialClips(), ...playClips(), ...rideClips(), ...reachClips(), ...shotClips(), ...lowClips(), ...selfClips(), ...serveClips()];
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
