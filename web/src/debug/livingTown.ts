/**
 * The living town's dev flag, read once from the URL:
 *
 * - `?ltFixture`: a Big world without `meta.backdrop` (a save from before the living town) gets the debug
 *   fixture's (`ltFixture.ts`). A world worldgen made carries its own, and the fixture leaves it alone.
 */

const params = typeof location === 'undefined' ? new URLSearchParams() : new URLSearchParams(location.search);

/** The debug fixture's backdrop for a Big world that has none (`?ltFixture`). */
export const LT_FIXTURE = params.has('ltFixture');
