/**
 * A town's public parking: marked bays in rows (car parks) off a straight stretch of street.
 */

// ---- The simulation's parking rules (sim-core `cars/parking.rs`, held to it by `web/tests/parkingRules.test.ts`) ----

/**
 * How near a driveway's mouth (where it meets its road, on the road's middle line) a bay's middle may not be (m): the
 * simulation parks no trip's car in a bay nearer (`by_a_mouth`), and a generated town lays none so.
 */
export const MOUTH_CLEAR = 7.0;
/** How near a driveway (the line from its inner end to its mouth) a bay's middle may not be (m), as `MOUTH_CLEAR`. */
export const DRIVE_CLEAR = 2.5;
/**
 * A bay: 3 m along the road by 6 m deep, side by side 3 m apart in a car park's row (m). A car parked in one stands at
 * its middle facing its yaw (nose in, away from the road), the pose the town file gives it.
 */
export const BAY_WIDE = 3.0;
export const BAY_DEEP = 6.0;
/** The lines painted round a bay, across and along it (m, about its middle): every car body parked in it is inside them. */
export const BAY_LINES: readonly [number, number] = [2.8, 5.6];
