/**
 * The living town's dev flags (docs/design/living-town.md), read once from the URL:
 *
 * - `?ltFixture`: a Big world without `meta.backdrop` gets the debug fixture's (`ltFixture.ts`), until
 *   worldgen writes its own.
 * - `?ltTown`: a stand-in for the camera stream's zoom (the orbit out to `TOWN_MAX`, the town view's
 *   state written each frame, the fog thinned and the far plane opened). `// lt-temp`: the integrator
 *   takes it out once the camera stream is merged.
 */

const params = typeof location === 'undefined' ? new URLSearchParams() : new URLSearchParams(location.search);

/** The debug fixture's backdrop for a Big world that has none (`?ltFixture`). */
export const LT_FIXTURE = params.has('ltFixture');

/** The stand-in town view camera (`?ltTown`). // lt-temp */
export const LT_TOWN = params.has('ltTown');
