/**
 * The living town's dev flag, read once from the URL:
 *
 * - `?ltFixture`: a Big world without `meta.backdrop` (a save from before the living town) gets the debug
 *   fixture's (`ltFixture.ts`). A world worldgen made carries its own, and the fixture leaves it alone.
 * - `?lotLooks`: a world without `meta.lotLooks` gets the fixture's for its public lots with walls, drawn closed as
 *   named places' lots (`ltFixture.ts` `applyLotLooksFixture`).
 */

const params = typeof location === 'undefined' ? new URLSearchParams() : new URLSearchParams(location.search);

/** The debug fixture's backdrop for a Big world that has none (`?ltFixture`). */
export const LT_FIXTURE = params.has('ltFixture');

/** The debug fixture's looks for the named places' lots of a world that has none (`?lotLooks`, `ltFixture.ts` `applyLotLooksFixture`). */
export const LOT_LOOKS_FIXTURE = params.has('lotLooks');
