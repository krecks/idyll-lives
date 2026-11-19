/**
 * Look tuning on the dev server: `?tune=lawn.tip:1.12;lawn.sky:0.85,0.95;look.vignette:0.8;key.13.skyColor:8FB4EE`
 * replaces named constants as their modules load, so one build is shot in several variants side by side
 * (`tools/bench/play.mjs --sheet`). A leaf module. Nothing is read in a production build or in the tests,
 * and a worker has no such flag in its address.
 *
 * The groups: `look` (`LOOK`'s own fields), `key.<hour>` (one time-of-day keyframe) and `night` (the night's,
 * all four) in `render/look.ts`; `lawn` in `pc/shaders/grass.ts`; `ground` (the lawn's map) in `shared/lawn.ts`;
 * `flowers` (the lawn's flowers' drifts) and `patches` (the patches of small flowers in them) in `shared/nature.ts`; `shade` (the ground's shade round what stands on it) in
 * `shared/groundShade.ts`; `clouds` (the clouds' shadows) in `pc/cloudShadows.ts`; `house` (the walls' tops, the tone of
 * walls and floors over a room) in `shared/house.ts`; `street` (worn asphalt, the pavement and its kerb, paving's sides) in
 * `shared/paving.ts`; `leaves` (the plants' leaves from above and from plant to plant) in `pc/nature.ts`; `room` (a lamplit
 * room's own light, and a room's shares of the sky's by day) in `shared/roomLight.ts`; `lampShadows` (the lamps' shadows) in `pc/lampShadows.ts`; `water` (the water's ripples, flow and foam) in
 * `pc/shaders/water.ts`; `contact` (the contact shading's pass) in `pc/post.ts` and `contactLight` (its share of the lamps' and the
 * sun's light) in `pc/shaders/softLook.ts`; `contactShadow` (the shadows shaped like what casts them) in
 * `shared/contactShadow.ts`; `floor` (the boards of a room with no floor laid) in `shared/house.ts`; `cars` (every
 * household car drawn as each body in turn) in `pc/cars.ts`.
 *
 * A surface of the asset manifest is tuned by its key and one of its own fields (`tunedEntry`, where `pc/materials.ts`
 * makes its material): `material.floor.wood.color:D9C8B4`, `material.floor.wood.uvScale:0.36`,
 * `material.floor.wood.texture:../floor_wood.calm/albedo.jpg` (a file named from the entry's own).
 */

/** `a.b:1;c.d:x` as path → value, both as written. */
export function parseTune(text: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const part of text.split(';')) {
    const i = part.indexOf(':');
    if (i > 0) out.set(part.slice(0, i).trim(), part.slice(i + 1).trim());
  }
  return out;
}

/**
 * `base` with `tune`'s entries under `group.` applied: a copy, or `base` itself when none apply. A value takes
 * the type of what it replaces: a number, a list of numbers (`0.85,0.95`), a colour (`8FB4EE` for `#8FB4EE`) or
 * text. An entry that names nothing is said on the console and left out.
 */
export function applyTune<T extends object>(group: string, base: T, tune: ReadonlyMap<string, string>): T {
  let out: T | null = null;
  for (const [key, raw] of tune) {
    if (!key.startsWith(`${group}.`)) continue;
    const path = key.slice(group.length + 1).split('.');
    out ??= structuredClone(base);
    let at: unknown = out;
    for (const k of path.slice(0, -1)) at = (at as Record<string, unknown> | undefined)?.[k];
    const last = path[path.length - 1];
    const was = (at as Record<string, unknown> | undefined)?.[last];
    if (was === undefined || (typeof was === 'object' && !Array.isArray(was))) {
      console.warn(`[tune] nothing to set at ${key}`);
      continue;
    }
    (at as Record<string, unknown>)[last] = Array.isArray(was) ? raw.split(',').map(Number) : typeof was === 'number' ? Number(raw) : typeof was === 'string' && was.startsWith('#') ? `#${raw.replace(/^#/, '')}` : raw;
  }
  return out ?? base;
}

let asked: ReadonlyMap<string, string> | undefined;

/** `base` as `?tune=` asks for it on the dev server; `base` itself otherwise. */
export function tuned<T extends object>(group: string, base: T): T {
  if (!import.meta.env.DEV || typeof location === 'undefined') return base;
  asked ??= parseTune(new URLSearchParams(location.search).get('tune') ?? '');
  return asked.size ? applyTune(group, base, asked) : base;
}

/**
 * `entry` (the asset manifest's, under `key`) with `tune`'s entries that name `key` and then one of the entry's own
 * fields: a copy, or `entry` itself when none do. (`material.floor.wood.color` is `material.floor.wood`'s colour, and
 * nothing of `material.floor`'s, which has no `wood`: said of neither.)
 */
export function applyEntryTune<T extends object>(key: string, entry: T, tune: ReadonlyMap<string, string>): T {
  const own = new Map<string, string>();
  for (const [path, raw] of tune) {
    if (path.startsWith(`${key}.`) && (entry as Record<string, unknown>)[path.slice(key.length + 1)] !== undefined) own.set(path, raw);
  }
  return own.size ? applyTune(key, entry, own) : entry;
}

/** A manifest entry as `?tune=` asks for it on the dev server; `entry` itself otherwise. */
export function tunedEntry<T extends object>(key: string, entry: T): T {
  if (!import.meta.env.DEV || typeof location === 'undefined') return entry;
  asked ??= parseTune(new URLSearchParams(location.search).get('tune') ?? '');
  return asked.size ? applyEntryTune(key, entry, asked) : entry;
}
