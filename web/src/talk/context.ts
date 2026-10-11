/**
 * Everything the game knows about a resident's moment, in words for the prompt (`prompt.ts`):
 * who they are (life stage, personality, job), when and where they are, how they feel, what's
 * on their mind, what's around them and who they're talking to, and what just happened. Read
 * from the UI's state (`game`), the content and the render snapshot; details (needs, feelings,
 * plans) exist only for the player's household and the inspected resident, the only ones who
 * speak anyway.
 */

import type { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import { objectCentre, type WorldStructure } from '../core/protocol';
import { game } from '../ui/state.svelte';
import { friendLabel } from '../ui/relationship';
import { goalText } from '../ui/story';
import { thoughtId } from '../voice/lines';
import type { LineRequest } from '../voice/writer';
import { whenText, type LineEvent, type Persona, type Situation, type TalkText } from './prompt';

/** Things within this many tiles (on the same storey) are "nearby". */
const NEAR = 3.5;
const MAX_NEARBY = 3;
/** Needs below this are mentioned ("hungry"). */
const LOW_NEED = 0.3;

export interface TalkSources {
  content: Content;
  /** The lot in view (its rooms), if any. */
  world: () => WorldStructure | null;
}

export interface Moment {
  persona: Persona;
  situation: Situation;
  event: LineEvent;
  /** The speaker's first name, and the other names the line may mention. */
  me: string;
  names: string[];
}

export function firstName(id: number): string {
  return (game.everyone.find((s) => s.id === id)?.name ?? game.roster.find((s) => s.id === id)?.name ?? '').split(' ')[0];
}

/** The moment of a line request, or null if it can't be told (babies, unknown residents, nothing to say). */
export function gatherMoment(text: TalkText, src: TalkSources, req: LineRequest, frame: FrameState): Moment | null {
  const { content } = src;
  const info = game.roster.find((s) => s.id === req.speaker);
  if (!info) return null;
  const view = game.sims.find((s) => s.id === req.speaker);
  const stage = view?.stage ?? info.stage ?? 'adult';
  if (stage === 'baby') return null;
  const me = firstName(req.speaker);
  const people = text.people[stage] ?? text.people.adult;
  const persona: Persona = {
    first: me,
    person: people?.[info.gender] ?? people?.[''] ?? '',
    job: view?.job ? (view.job.careerLabel && view.job.careerLabel !== view.job.title ? `${view.job.title} (${view.job.careerLabel})` : view.job.title) : undefined,
    school: !view?.job && (stage === 'child' || stage === 'teen'),
    personality: [
      ...info.traits.map((t) => text.traits[t] ?? content.trait(t)?.label.toLowerCase()).filter((t): t is string => !!t),
      ...info.perks.map((p) => text.perks[p]).filter((p): p is string => !!p),
    ],
  };

  const { curr, layout } = frame;
  const k = layout.sim;
  const o = layout.headerLen + req.row * layout.simStride;
  const x = curr[o + k.x];
  const z = curr[o + k.z];

  const feeling: string[] = [];
  const emotion = content.emotion(view?.emotion ?? null)?.label.toLowerCase();
  if (emotion) feeling.push(emotion);
  for (const f of [...(view?.feelings ?? [])].sort((a, b) => Math.abs(b.mood) - Math.abs(a.mood)).slice(0, 2)) feeling.push(f.label.toLowerCase());
  content.needs.forEach((n, i) => {
    const v = view?.needs[i];
    if (v !== undefined && v < LOW_NEED && text.needs[n.id]) feeling.push(text.needs[n.id]);
  });

  const mind: string[] = [];
  const plan = view?.plan;
  for (const g of plan?.goals.slice(0, 1) ?? []) mind.push(lowerFirst(goalText(g.def, g.target, g.skill, g.category)));
  for (const [activity] of plan?.wishes.slice(0, 1) ?? []) {
    const a = content.activity(activity);
    if (a) mind.push(`a place to ${a.label.toLowerCase()}`);
  }

  const using = curr[o + k.object];
  // Things in the same room (walls between: not nearby), where the lot in view tells rooms.
  const lot = src.world();
  const roomAt = (px: number, pz: number) => (lot ? lot.rooms[Math.floor(pz) * lot.width + Math.floor(px)] : undefined);
  const here = roomAt(x, z);
  const nearby = [
    ...new Set(
      game.objects
        .filter((obj) => obj.id !== using)
        .filter((obj) => {
          if (here === undefined) return true;
          const c = objectCentre(obj);
          return roomAt(c.x, c.z) === here;
        })
        .map((obj) => {
          const c = objectCentre(obj);
          return { name: content.object(obj.def)?.name, d: Math.hypot(c.x - x, c.z - z) };
        })
        .filter((n): n is { name: string; d: number } => !!n.name && n.d <= NEAR)
        .sort((a, b) => a.d - b.d)
        .map((n) => lowerFirst(n.name)),
    ),
  ].slice(0, MAX_NEARBY);

  const held = k.held === undefined ? 0 : curr[o + k.held] | 0;
  const heldId = held > 0 ? layout.items?.[held - 1] : undefined;
  const holding = heldId ? content.item(heldId)?.label.toLowerCase() : undefined;

  const situation: Situation = {
    when: whenText(text, game.weekday, game.minute),
    place: placeOf(text, src, view?.plot ?? null, info.household, x, z),
    feeling: feeling.slice(0, 4),
    mind,
    nearby,
    holding,
  };
  const names: string[] = [];
  if (req.partner >= 0) {
    const name = firstName(req.partner);
    if (name) {
      names.push(name);
      situation.partner = { name, relation: relationOf(text, req.speaker, req.partner) };
    }
  }

  const event = eventOf(src, req, frame, situation.partner?.name ?? '');
  return event ? { persona, situation, event, me, names } : null;
}

/** What happened, in words. */
function eventOf({ content }: TalkSources, req: LineRequest, frame: FrameState, other: string): LineEvent | null {
  switch (req.kind) {
    case 'social':
      return req.social && req.phase ? { kind: 'social', social: req.social, phase: req.phase, other: other || 'someone' } : null;
    case 'use': {
      const def = req.object ? content.object(req.object) : undefined;
      const interaction = def?.interactions.find((i) => i.id === req.interaction);
      const { curr, layout } = frame;
      const k = layout.sim;
      const recipeIndex = k.recipe === undefined ? 0 : curr[layout.headerLen + req.row * layout.simStride + k.recipe] | 0;
      const recipe = recipeIndex > 0 ? content.recipes.find((r) => r.id === layout.recipes?.[recipeIndex - 1])?.label : undefined;
      const what = interaction?.label ?? req.tag;
      if (!what) return null;
      const action = [what.toLowerCase(), recipe ? `(${recipe.toLowerCase()})` : def ? `(${lowerFirst(def.name)})` : ''].filter(Boolean).join(' ');
      return { kind: 'use', action };
    }
    case 'emotion': {
      const e = req.emotion ? content.emotions.find((x) => x.id === req.emotion) : undefined;
      return e ? { kind: 'emotion', emotion: e.label.toLowerCase() } : null;
    }
    case 'thought': {
      const id = req.thought ? thoughtId(req.thought) : undefined;
      return id ? { kind: 'thought', thought: id, subject: thoughtSubject(content, req.thought!, req.thoughtSubject ?? 0) } : null;
    }
  }
}

/** A thought's subject in words: the activity, goal, room, broken thing. */
function thoughtSubject(content: Content, kind: number, subject: number): string {
  switch (kind) {
    case 4:
      return lowerFirst(goalText(subject));
    case 5:
    case 6: {
      const room = Math.floor(subject / 8);
      return room === 0 ? 'garden' : (content.roomKinds[room - 1]?.label?.toLowerCase() ?? 'room');
    }
    case 7:
      return lowerFirst(content.objectList[subject]?.name ?? 'thing');
    case 8:
    case 9:
    case 10:
      return '';
    default:
      return content.activities[subject]?.label.toLowerCase() ?? 'do it';
  }
}

/** Where they are: at home (and which room), visiting someone, at a public place or out. */
function placeOf(text: TalkText, src: TalkSources, plot: number | null, household: number, x: number, z: number): string {
  const w = text.situation;
  if (plot === null) return w.out;
  const home = game.households.find((h) => h.id === household)?.plot;
  if (plot === home) {
    const room = roomName(src, x, z);
    return room ? w.homeRoom.replace('{subject}', room) : w.home;
  }
  const host = game.households.find((h) => h.plot === plot);
  if (host) return w.visiting.replace('{subject}', `the ${host.name} home`);
  const place = game.plots.find((p) => p.id === plot);
  return place?.public ? `at ${place.name}` : w.out;
}

/** The kind of room at a spot of the player's home (the only one with room kinds), if any. */
function roomName({ content, world }: TalkSources, x: number, z: number): string | null {
  const lot = world();
  if (!lot) return null;
  const tile = Math.floor(z) * lot.width + Math.floor(x);
  const id = lot.rooms[tile];
  if (id === undefined) return null;
  const room = game.rooms.find((r) => r.id === id);
  if (!room) return null;
  if (room.garden) return 'garden';
  return room.kind === undefined ? null : (content.roomKinds[room.kind]?.label?.toLowerCase() ?? null);
}

/** What `b` is to `a`: family, partner, or how friendly they are. */
function relationOf(text: TalkText, a: number, b: number): string {
  const r = game.relationships.find((x) => x.a === a && x.b === b);
  const back = game.relationships.find((x) => x.a === b && x.b === a);
  const rel = text.relations;
  if (!r) return rel.stranger;
  if (r.partners) return rel.partner;
  if (r.kin) return rel[r.kin] ?? rel.friend;
  const label = friendLabel(r.friendship, back?.friendship ?? r.friendship);
  const key = { Enemy: 'enemy', Disliked: 'disliked', 'Best friend': 'bestFriend', 'Good friend': 'goodFriend', Friend: 'friend', Acquaintance: 'acquaintance' }[label];
  return (key && rel[key]) || rel.acquaintance;
}

function lowerFirst(s: string): string {
  // Keep names in capitals ("TV"); lower a capital followed by lower case.
  return /^[A-Z][a-z]/.test(s) ? s[0].toLowerCase() + s.slice(1) : s;
}
