/**
 * The new game's state in a debug report (F8 on a New game step, `host.basicReport`): the world's
 * code, step 1's land and step 2's layout, so `?debugReport=<name>` (dev server) opens New game
 * again at that step with that land and layout (`App.svelte` `openDebugReport`). The dev server
 * writes it as `newgame.json` next to the report; a download carries it as the payload's `newGame`.
 *
 * The drafts are a graph of plain data with typed arrays in it, objects shared between drafts, and
 * the land's height field (up to 480 × 480 samples). `pack` writes it as JSON: each object met
 * more than once once (`$id`, then `$ref`), a height field as the town file codes it (`encodeLand`),
 * other typed arrays in base64, and each such block once however often it recurs (`blobs`).
 * `unpack` gives the same graph back, the same objects shared.
 */

import type { Pt } from '../game/roads';
import { TownHistory } from '../game/edit/history';
import { decodeLand, encodeLand, type Land } from '../game/land';
import { writeCode } from '../game/worldCode';
import type { LandFile } from '../core/protocol';
import type { Layout, MadeLand, Screen } from '../ui/app.svelte';
import type { Chosen } from '../ui/newGame';

/** What of `app` (`app.svelte.ts`) the report keeps. */
export interface NewGameState {
  screen: Screen;
  code: Chosen | null;
  land: MadeLand | null;
  site: Pt | null;
  townName: string | null;
  buildSeed: number;
  layout: Layout | null;
}

/** What a report gives back: the step New game opens at (later steps open at the streets, made again from their keys), and `app`'s fields. */
export interface RestoredNewGame extends Omit<NewGameState, 'screen'> {
  step: 'land' | 'streets';
}

/** The screens New game is set up on (`app.svelte.ts` `Screen`): a report on any of them carries its state. */
const STEPS: readonly Screen[] = ['land', 'streets', 'town', 'household', 'create', 'home'];

/** A layout's fields as packed: its undo history left out but for the plan as it stands (`pending`, where it is ahead of `plan`). */
type PackedLayout = Omit<Layout, 'history'> & { pending?: Layout['plan'] };

interface NewGameFile {
  v: 1;
  /** The screen F8 was pressed on. */
  step: Screen;
  /** The world's code as the player would copy it (`writeCode`), and as `app` keeps it. */
  code: string | null;
  chosen: Chosen | null;
  site: Pt | null;
  townName: string | null;
  buildSeed: number;
  /** `land`: step 1's land as taken on; `shown`: step 1's as on screen (its shaping), where it differs; `layout`: step 2's. */
  state: Packed;
}

// --- Step 1's land as on screen ------------------------------------------------------------------

/** Step 1's land as Next would take it on (`CreateLand.svelte`), while step 1 is open. */
let shownLand: (() => MadeLand | null) | null = null;

/** Step 1 tells the report its land as on screen (shaped since it was taken on, or not yet taken on); the returned function stops it. */
export function reportShownLand(get: () => MadeLand | null): () => void {
  shownLand = get;
  return () => {
    if (shownLand === get) shownLand = null;
  };
}

// --- The report ---------------------------------------------------------------------------------

/** The new game's part of a report (JSON), or null off New game's steps (the main menu, the game). */
export function newGameReport(s: NewGameState): string | null {
  if (!STEPS.includes(s.screen)) return null;
  let shown: MadeLand | null = null;
  if (s.screen === 'land') {
    try {
      shown = shownLand?.() ?? null;
    } catch (err) {
      console.warn('[debug] step 1 land for the report failed', err);
    }
    if (shown && shown.key === s.land?.key) shown = null;
  }
  const file: NewGameFile = {
    v: 1,
    step: s.screen,
    code: s.code ? writeCode(s.code) : null,
    chosen: s.code,
    site: s.site,
    townName: s.townName,
    buildSeed: s.buildSeed,
    state: pack({ land: s.land, shown, layout: s.layout ? packLayout(s.layout) : null }),
  };
  return JSON.stringify(file);
}

/** A report's new game part read back (`newGameReport`): what `app` is given to open New game again. */
export function readNewGameReport(text: string): RestoredNewGame {
  const file = JSON.parse(text) as NewGameFile;
  if (file.v !== 1 || !file.state) throw new Error("That isn't a new game's report");
  const { land, shown, layout } = unpack(file.state) as { land: MadeLand | null; shown: MadeLand | null; layout: PackedLayout | null };
  const step = file.step === 'land' ? 'land' : 'streets';
  // Step 1's land as on screen is what Next would take on: taken on, the town's site goes with the land it was moved on.
  const onScreen = step === 'land' && shown;
  return {
    step,
    code: file.chosen,
    land: onScreen ? shown : land,
    site: onScreen ? null : file.site,
    townName: file.townName,
    buildSeed: file.buildSeed,
    layout: layout ? unpackLayout(layout) : null,
  };
}

function packLayout(layout: Layout): PackedLayout {
  const { history, ...rest } = layout;
  const now = history?.draft;
  return { ...rest, ...(now && now !== (layout.plan ?? layout.proposal.plan) ? { pending: now } : {}) };
}

/** A packed layout with its edits not yet settled as one edit ahead of its plan (the editor settles it on attaching). */
function unpackLayout(packed: PackedLayout): Layout {
  const { pending, ...layout } = packed;
  if (!pending) return layout;
  const history = new TownHistory(layout.plan ?? layout.proposal.plan);
  history.push(pending);
  return { ...layout, history };
}

// --- The graph's packing -------------------------------------------------------------------------

/** A packed graph: its root, and the blocks (height fields, typed arrays) it refers to by index. */
export interface Packed {
  root: unknown;
  blobs: unknown[];
}

type TypedArray = Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array;
const TYPED = { Int8Array, Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array, Int32Array, Uint32Array, Float32Array, Float64Array } as const;
type TypedName = keyof typeof TYPED;

const LAND_KEYS = new Set(['cols', 'rows', 'h', 'wl', 'cover', 'seeded', 'edited']);

/** A height field (`land.ts` `Land`), with nothing else on it. */
function isLand(v: object): v is Land {
  const l = v as Partial<Land>;
  return (
    l.h instanceof Int16Array &&
    l.wl instanceof Int16Array &&
    l.cover instanceof Uint8Array &&
    typeof l.cols === 'number' &&
    typeof l.rows === 'number' &&
    l.h.length === l.cols * l.rows &&
    l.wl.length === l.h.length &&
    (l.seeded === 'drawn' || l.seeded === 'follow') &&
    Object.keys(v).every((k) => LAND_KEYS.has(k))
  );
}

function typedName(v: object): TypedName | null {
  if (!ArrayBuffer.isView(v) || v instanceof DataView) return null;
  const name = v.constructor.name as TypedName;
  return Object.hasOwn(TYPED, name) ? name : null;
}

function base64(bytes: Uint8Array): string {
  let text = '';
  for (let k = 0; k < bytes.length; k += 0x2000) text += String.fromCharCode(...bytes.subarray(k, Math.min(bytes.length, k + 0x2000)));
  return btoa(text);
}

function fromBase64(code: string): Uint8Array<ArrayBuffer> {
  const text = atob(code);
  const bytes = new Uint8Array(text.length);
  for (let k = 0; k < text.length; k++) bytes[k] = text.charCodeAt(k);
  return bytes;
}

/** `value` as plain JSON (see the file's comment). */
export function pack(value: unknown): Packed {
  // Pass 1: how often each object is met (an object's insides are walked the first time only).
  const seen = new Map<object, number>();
  const count = (v: unknown): void => {
    if (typeof v !== 'object' || v === null) return;
    const n = seen.get(v) ?? 0;
    seen.set(v, n + 1);
    if (n || typedName(v) || isLand(v)) return;
    if (Array.isArray(v) || v instanceof Set) for (const x of v) count(x);
    else if (v instanceof Map) for (const [k, x] of v) (count(k), count(x));
    else for (const x of Object.values(v)) count(x);
  };
  count(value);

  const ids = new Map<object, number>();
  const blobs: unknown[] = [];
  const blobOf = new Map<string, number>();
  const blob = (key: string, data: unknown) => {
    let at = blobOf.get(key);
    if (at === undefined) {
      at = blobs.push(data) - 1;
      blobOf.set(key, at);
    }
    return at;
  };
  const emit = (v: unknown): unknown => {
    if (v === undefined) return { $u: 1 };
    if (typeof v === 'number') return Number.isFinite(v) ? v : { $n: String(v) };
    if (typeof v === 'bigint' || typeof v === 'function' || typeof v === 'symbol') throw new Error(`can't pack a ${typeof v}`);
    if (typeof v !== 'object' || v === null) return v;
    const known = ids.get(v);
    if (known !== undefined) return { $ref: known };
    const id: { $id?: number } = {};
    if ((seen.get(v) ?? 0) > 1) {
      id.$id = ids.size;
      ids.set(v, id.$id);
    }
    const typed = typedName(v);
    if (typed) {
      const view = v as TypedArray;
      const code = base64(new Uint8Array(view.buffer, view.byteOffset, view.byteLength));
      return { ...id, $ta: typed, $b: blob(`${typed}:${code}`, code) };
    }
    if (isLand(v)) {
      const file = encodeLand(v);
      return { ...id, $land: blob(`land:${JSON.stringify(file)}`, file), edited: v.edited };
    }
    if (Array.isArray(v)) return id.$id === undefined ? v.map(emit) : { ...id, $a: v.map(emit) };
    if (v instanceof Set) return { ...id, $set: [...v].map(emit) };
    if (v instanceof Map) return { ...id, $map: [...v].map(([k, x]) => [emit(k), emit(x)]) };
    const out: Record<string, unknown> = { ...id };
    for (const [k, x] of Object.entries(v)) if (x !== undefined) out[k] = emit(x);
    return out;
  };
  return { root: emit(value), blobs };
}

/** A graph `pack` packed, back as it was. */
export function unpack(packed: Packed): unknown {
  const { blobs } = packed;
  const ids: unknown[] = [];
  const keep = <T>(o: Record<string, unknown>, v: T): T => {
    if (typeof o.$id === 'number') ids[o.$id] = v;
    return v;
  };
  const read = (v: unknown): unknown => {
    if (typeof v !== 'object' || v === null) return v;
    if (Array.isArray(v)) return v.map(read);
    const o = v as Record<string, unknown>;
    if ('$ref' in o) return ids[o.$ref as number];
    if ('$u' in o) return undefined;
    if ('$n' in o) return Number(o.$n);
    if ('$ta' in o) {
      const bytes = fromBase64(blobs[o.$b as number] as string);
      const Kind = TYPED[o.$ta as TypedName];
      return keep(o, new Kind(bytes.buffer, 0, bytes.byteLength / Kind.BYTES_PER_ELEMENT));
    }
    if ('$land' in o) return keep(o, { ...decodeLand(blobs[o.$land as number] as LandFile), edited: !!o.edited });
    if ('$a' in o) {
      const out = keep(o, [] as unknown[]);
      for (const x of o.$a as unknown[]) out.push(read(x));
      return out;
    }
    if ('$set' in o) {
      const out = keep(o, new Set<unknown>());
      for (const x of o.$set as unknown[]) out.add(read(x));
      return out;
    }
    if ('$map' in o) {
      const out = keep(o, new Map<unknown, unknown>());
      for (const [k, x] of o.$map as [unknown, unknown][]) out.set(read(k), read(x));
      return out;
    }
    const out = keep(o, {} as Record<string, unknown>);
    for (const [k, x] of Object.entries(o)) if (k !== '$id') out[k] = read(x);
    return out;
  };
  return read(packed.root);
}
