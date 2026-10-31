/**
 * Socials' body language (`residents.ts` `choose`): the snapshot's animation codes, the gestures per social, and
 * how hugs, kisses, refusals and punches fit two residents of any heights.
 */



/** Snapshot animation codes (see `social::ANIMATIONS` in sim-core). */
export const Anim = {
  None: 0,
  Talk: 1,
  Laugh: 2,
  Flirt: 3,
  Argue: 4,
  Fight: 5,
  Hug: 6,
  Kiss: 7,
  Wave: 8,
} as const;
/**
 * A social's body language (`choose`; the clips in characters/clips/social.ts). `speak`: the one who
 * started it, on their turn (default `talk`); `answer`: the other on theirs (default `talk`); `listen`:
 * either while the other speaks (default the idle, with a nod or a head shake now and then once it's
 * going well or badly). Once the outcome shows (snapshot `outcome`): `good` / `bad`, the other's (and
 * those listening in) for the rest of it when it went well / badly, and `flop`, the starter's when it
 * went badly (a joke nobody laughs at, a flirt turned down). A `good` laugh is shared: the teller
 * laughs along between their turns.
 */
export interface Gesture {
  speak?: string;
  answer?: string;
  listen?: string;
  good?: string;
  bad?: string;
  flop?: string;
}
/** Per animation code, for socials without their own (`GESTURES`; packs'). */
export const ANIM_GESTURES: Record<number, Gesture> = {
  [Anim.Laugh]: { good: 'laugh', bad: 'foldArms', flop: 'shrug' },
  [Anim.Flirt]: { speak: 'flirt', bad: 'foldArms', flop: 'rebuffed' },
  [Anim.Argue]: { speak: 'argue', answer: 'argue', listen: 'no' },
};
/** Per social (content `socials` id), over its animation code's. */
export const GESTURES: Record<string, Gesture> = {
  introduce: { speak: 'introduce', answer: 'introduce' },
  joke: { speak: 'joke' },
  compliment: { speak: 'compliment', bad: 'foldArms', flop: 'shrug' },
  deepTalk: { speak: 'deepTalk', answer: 'deepTalk' },
  flirt: { answer: 'flirt' },
  sweetTalk: { speak: 'sweetTalk', answer: 'sweetTalk' },
  askPartner: { speak: 'askPartner', bad: 'refuse', flop: 'rebuffed' },
  breakUp: { speak: 'breakUp', answer: 'argue', listen: 'no' },
  insult: { speak: 'insult', listen: 'no' },
  prank: { speak: 'prank' },
  apologize: { speak: 'apologize', bad: 'foldArms', flop: 'rebuffed' },
};
/** A social's gestures (`GESTURES`) over its animation code's (`ANIM_GESTURES`), per code then social id, merged once. */
export const MERGED_GESTURES = new Map<number, Map<string, Gesture>>();
export function gesturesOf(anim: number, social: string): Gesture {
  let bySocial = MERGED_GESTURES.get(anim);
  if (!bySocial) MERGED_GESTURES.set(anim, (bySocial = new Map()));
  let g = bySocial.get(social);
  if (!g) bySocial.set(social, (g = { ...ANIM_GESTURES[anim], ...GESTURES[social] }));
  return g;
}
/**
 * A refused hug or kiss (`Anim.Hug`, `Anim.Kiss`, snapshot `outcome` 2, shown from the start): the one asked
 * leans away with a hand up (`refuse`) and steps back `back` m; the one who asked stays where they stood,
 * the shoulders dropped (`rebuffed`).
 */
export const REFUSED = { back: 0.14 };
/**
 * A kiss between two of any heights (`Anim.Kiss`), per metre the one is taller than the other (rad):
 * `bend`, the taller's back bent over; `nod`, their head bowed; `lift`, the shorter's face lifted (the neck,
 * the head and the upper back); each at most `most`. `lean`: how far both lean in. The arms: the taller's
 * raised `waist` round the other's waist (elbows `waistWrap`), the shorter's raised `neck` up round the
 * other's shoulders (elbows `neckWrap`, `hugArms`).
 */
export const KISS_FIT = { bend: 0.7, nod: 1.3, lift: 1.8, most: 0.45, lean: 0.06, waist: 0.75, waistWrap: 1.35, neck: 1.4, neckWrap: 1.25 };
/**
 * Punches aimed at the other's head at any two heights (`Anim.Fight`, the attacker): the fist goes `head` of
 * the difference in height lower (the clips aim at one's own face, at about that share of the height), at
 * `reach` m (body scale 1); of that angle (at most `most` rad), `bend` is bent down at the back over a
 * shorter one and `straighten` straightened up to a taller one, the arms doing the rest.
 */
export const AIM = { head: 0.85, reach: 0.55, most: 0.7, bend: 0.45, straighten: 0.3 };
/**
 * Hugs and kisses (`Anim.Hug`, `Anim.Kiss`): the two step in from where sim-core stands them (about 0.55 m
 * apart, middle to middle) until their fronts meet. `chest`: how far the chest stands out in front of the
 * body's middle, as a share of its height (grown-ups' bodies: 0.13 m of 1.77 m; times its build); `kiss`: the
 * gap a kiss keeps between the chests (m; the faces meet, the bodies lean in); `most`: the furthest one steps
 * (m). `shoulder`: the shoulders' height, as a share of the body's; `lower`: how much (rad) a hugger lowers the
 * arms for each metre the other's shoulders are below theirs (a grown-up's round a child), down to `low`, and
 * `bend`: how far (rad) they bend down over them per metre. `wrap`: the elbows' bend round the other's back
 * (rad; `wrapSmall` more per metre lower, round a child's), with the hands behind it, not out past it;
 * `aside`: how far (rad) each leans the head to their own right, past the other's.
 */
export const EMBRACE = { chest: 0.074, kiss: 0.03, most: 0.45, shoulder: 0.79, lower: 1.4, low: 0.35, bend: 0.9, wrap: 1.45, wrapSmall: 1.2, aside: 0.1 };
