/**
 * A machine with 8 GB of memory or less (`navigator.deviceMemory`, which the browser rounds down to a
 * power of two: an 8 GB machine says 8, a 16 GB one 16), or `?lowmem` on the dev server to try it on a
 * bigger one. There the game holds less at once:
 * - the menus' town is let go of as soon as the loading screen covers it, before the game's world is
 *   built, and the game fades in rather than the camera flying in (`game/host.ts`);
 * - the voices and the language model load when they're first wanted, not as the page boots
 *   (`voice/service.svelte.ts`, `talk/talk.svelte.ts`).
 */

/** The most memory (GB, as `navigator.deviceMemory` says it) a machine has and still counts as low on memory. */
const LOW_MEMORY_GB = 8;

// (Read where there's a page: a test's stand-in has neither.)
const memory = (globalThis.navigator as (Navigator & { deviceMemory?: number }) | undefined)?.deviceMemory;

export const LOW_MEMORY = (import.meta.env.DEV && new URLSearchParams(globalThis.location?.search ?? '').has('lowmem')) || (memory !== undefined && memory <= LOW_MEMORY_GB);
