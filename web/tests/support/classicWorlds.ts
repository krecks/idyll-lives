// The classic worlds: the eight kinds a world was before places and options (small, medium and
// dense towns, a farm, a seaside village, a mountain hamlet, a lakeside town, a forest clearing),
// each with the tables it was laid out from. Tests that pin an exact world (its name, a road of
// it, a plot) make it from these (`macro.ts` `generateFromSpec`), so a change to the places leaves
// them be; `classicWorlds.slow.test.ts` checks they still come out byte for byte as they did.
export { CLASSIC, type ClassicKind } from '../../src/game/world';
