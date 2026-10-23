/**
 * A house's wall sizes, shared by the house builder (`house.ts`, which re-exports them) and what
 * stands against its walls (`models.ts`) without pulling in the whole builder.
 */

/** A storey's wall height (m). */
export const WALL_HEIGHT = 2.8;
/** Half a wall's thickness (m; sim-core WALL_THICKNESS / 2). */
export const WALL_HALF = 0.07;
