/**
 * The land brushes as the palette offers them (`LandPalette.svelte`, in step 1 and in Build mode's
 * Land tab): the tools in their four groups with what each does, the brush in hand (its size,
 * strength and, for the mountains, height) and how it turns into the editor's `Brush`, the
 * cursor's look, and what a stroke costs as the cursor tag says it.
 */

import type { LandCursor } from '../../render/types';
import { aimOf, isPoint, MOUNTAIN_HEIGHT, POINT, pointGrid, TOOL_KIND, type Brush, type LandTool } from '../../game/landBrush';

export { POINT };

export type LandGroup = 'shape' | 'water' | 'cover' | 'mountains';

export interface LandToolInfo {
  id: LandTool;
  label: string;
  /** What it does, for the hint line when the editor has nothing to say. */
  does: string;
}

/** What the palette and the cursor tag show. */
export interface LandState {
  surveying: boolean;
  ready: boolean;
  canUndo: boolean;
  canRedo: boolean;
  /** The land differs from how it was surveyed (it can be reset). */
  changed: boolean;
  hint: string | null;
  /** A stroke is under way, and what it has cost so far. */
  stroking: boolean;
  cost: number;
  /** The brush would do nothing where the pointer is (built land, or water where it may not go). */
  refused: boolean;
  /** A command is on its way to the simulation. */
  busy: boolean;
}

export const NO_LAND: LandState = { surveying: false, ready: false, canUndo: false, canRedo: false, changed: false, hint: null, stroking: false, cost: 0, refused: false, busy: false };

/** The palette's groups, in order, and their tools. */
export const LAND_GROUPS: readonly { id: LandGroup; label: string; tools: readonly LandToolInfo[] }[] = [
  {
    id: 'shape',
    label: 'Shape',
    tools: [
      { id: 'raise', label: 'Raise', does: 'Hold and drag to raise the ground.' },
      { id: 'lower', label: 'Lower', does: 'Hold and drag to lower the ground.' },
      { id: 'smooth', label: 'Smooth', does: 'Drag over bumps to even them out.' },
      { id: 'flatten', label: 'Flatten', does: 'Drag to bring the ground to the height where you pressed.' },
    ],
  },
  {
    id: 'water',
    label: 'Water',
    tools: [
      { id: 'lake', label: 'Lake', does: 'Paint a pond or a lake. Start on the sea to push the coast back.' },
      { id: 'river', label: 'River', does: 'Drag a river across the land. It runs downhill, in rapids where the land falls.' },
      { id: 'dry', label: 'Dry', does: 'Drag over water to fill it in.' },
    ],
  },
  {
    id: 'cover',
    label: 'Cover',
    tools: [
      { id: 'woods', label: 'Woods', does: 'Paint woods. The trees grow when you let go.' },
      { id: 'meadow', label: 'Meadow', does: 'Paint meadow and wild flowers.' },
      { id: 'fields', label: 'Fields', does: 'Paint farm fields.' },
      { id: 'clear', label: 'Clear', does: 'Clear woods, meadow or fields back to grass.' },
    ],
  },
  {
    id: 'mountains',
    label: 'Mountains',
    tools: [
      { id: 'peak', label: 'Peak', does: 'Click or drag to place a peak. It rises when you let go.' },
      { id: 'range', label: 'Range', does: 'Drag a line of mountains. They rise when you let go.' },
      { id: 'level', label: 'Level', does: 'Drag to wear the land down towards its surroundings.' },
    ],
  },
];

const INFO = new Map(LAND_GROUPS.flatMap((g) => g.tools.map((t) => [t.id, { ...t, group: g.id }] as const)));

export function toolInfo(tool: LandTool): LandToolInfo & { group: LandGroup } {
  return INFO.get(tool)!;
}

export const groupOf = (tool: LandTool): LandGroup => toolInfo(tool).group;

/**
 * The brush's size (its width across, m) each group allows, and a group's size to begin with: down
 * to one sample for the shape tools, a pond of one sample, one cover cell (`POINT` and under is a
 * point brush).
 */
export const SIZES: Record<LandGroup, { min: number; max: number; start: number }> = {
  shape: { min: 1, max: 120, start: 30 },
  water: { min: 2, max: 120, start: 24 },
  cover: { min: 4, max: 120, start: 40 },
  mountains: { min: 60, max: 400, start: 180 },
};
/** A river is a line 4 to 30 m wide. */
const RIVER_SIZE = { min: 4, max: 30 };
/** The mountains' height (m) the slider allows. */
export const HEIGHTS = { min: 10, max: 120 };

/** The size range of a tool (m across). */
export function sizeRange(tool: LandTool): { min: number; max: number } {
  return tool === 'river' ? RIVER_SIZE : SIZES[groupOf(tool)];
}

/**
 * The size (whole m across) at slider position `t` (0..1) over `range`, on a log scale: each step
 * along the slider makes the brush the same share bigger, so the small sizes get as much of it as
 * the large ones. `sliderAt` is the other way: `sizeAt(sliderAt(size))` is `size` for every whole
 * size in the range.
 */
export function sizeAt(t: number, range: { min: number; max: number }): number {
  const size = Math.round(range.min * (range.max / range.min) ** Math.min(1, Math.max(0, t)));
  return Math.min(range.max, Math.max(range.min, size));
}

export function sliderAt(size: number, range: { min: number; max: number }): number {
  const s = Math.min(range.max, Math.max(range.min, size));
  return range.max > range.min ? Math.log(s / range.min) / Math.log(range.max / range.min) : 0;
}

/** A size step from the keyboard (`dir` up or down): about a tenth bigger or smaller, and at least a metre, within `range`. */
export function stepSize(size: number, dir: 1 | -1, range: { min: number; max: number }): number {
  const next = dir > 0 ? Math.max(size + 1, Math.round(size * 1.1)) : Math.min(size - 1, Math.round(size / 1.1));
  return Math.min(range.max, Math.max(range.min, next));
}

/** The size as the palette writes it: "one point" for a point brush, else whole metres ("120 m"). */
export const sizeText = (size: number) => (size <= POINT ? 'one point' : `${Math.round(size)} m`);

/** The brush in hand, as the palette sets it: the tool, and each group's size (m across), the strength (0..1), the mountains' height (m). */
export interface BrushState {
  tool: LandTool;
  sizes: Record<LandGroup, number>;
  strength: number;
  height: number;
}

export function startBrush(tool: LandTool = 'raise'): BrushState {
  return { tool, sizes: { shape: SIZES.shape.start, water: SIZES.water.start, cover: SIZES.cover.start, mountains: SIZES.mountains.start }, strength: 0.5, height: MOUNTAIN_HEIGHT };
}

/** The brush's size (m across) for the tool in hand, within the tool's range. */
export function sizeOf(state: BrushState): number {
  const { min, max } = sizeRange(state.tool);
  return Math.min(max, Math.max(min, state.sizes[groupOf(state.tool)]));
}

/** The editor's brush for the state (`radius` half the size). */
export function brushOf(state: BrushState): Brush {
  const mountain = state.tool === 'peak' || state.tool === 'range';
  return { tool: state.tool, radius: sizeOf(state) / 2, strength: Math.min(1, Math.max(0, state.strength)), ...(mountain ? { height: state.height } : {}) };
}

/** Whether the tool has a strength (the cover tools paint or not; the peak and range have a height instead). */
export const hasStrength = (tool: LandTool) => TOOL_KIND[tool] !== 'cover' && tool !== 'dry' && tool !== 'peak' && tool !== 'range';
export const hasHeight = (tool: LandTool) => tool === 'peak' || tool === 'range';

/**
 * Where the falloff `(1 - (r/R)²)²` is a half: the inner ring of the cursor (within it the brush
 * works at least at half strength, and the cover tools paint).
 */
export const INNER = Math.sqrt(1 - Math.SQRT1_2);

/** The cursor's tone for the tool, or `bad` where it would do nothing. */
export function toneOf(tool: LandTool, refused: boolean): LandCursor['tone'] {
  if (refused) return 'bad';
  switch (TOOL_KIND[tool]) {
    case 'water':
      return 'water';
    case 'cover':
      return 'cover';
    default:
      return tool === 'lower' || tool === 'level' || tool === 'smooth' ? 'down' : 'up';
  }
}

/**
 * The cursor for a brush with the pointer at town point (`x`, `z`). A point brush's sits on the
 * grid point it would change (`aimOf`), a ring as wide as that sample (a cover tool's: that cell),
 * with no inner ring: the whole of it works at full strength.
 */
export function cursorOf(brush: Brush, x: number, z: number, refused: boolean): LandCursor {
  const tone = toneOf(brush.tool, refused);
  if (!isPoint(brush)) return { x, z, radius: brush.radius, inner: brush.radius * INNER, tone };
  const [px, pz] = aimOf(brush, x, z);
  return { x: px, z: pz, radius: pointGrid(brush.tool).unit / 2, inner: 0, tone };
}

/** Whole dollars ($1,234). */
const dollars = (n: number) => `$${Math.round(n).toLocaleString('en-US')}`;

/**
 * What the cursor tag says of a stroke's cost: free in Creative; the price (or the refund) in
 * Living, and how much short where the funds don't reach. `money` writes an amount (the HUD's).
 */
export function costLabel(cost: number, creative: boolean, funds: number, money: (amount: number) => string = dollars): { text: string; bad: boolean } {
  if (creative) return { text: 'Free', bad: false };
  if (cost < 0) return { text: `${money(-cost)} back`, bad: false };
  if (cost > funds) return { text: `${money(cost - funds)} short`, bad: true };
  return { text: money(cost), bad: false };
}

/**
 * Whether a stroke costing `cost` so far should stop: in Living, once `ahead` more stamps at its
 * average so far (`stamps` of them) would take it past the funds (a frame may stamp a few times).
 */
export function stopsAtFunds(cost: number, stamps: number, funds: number, creative: boolean, ahead = 1): boolean {
  if (creative || cost <= 0) return false;
  const each = stamps > 0 ? cost / stamps : 0;
  return cost + each * ahead > funds;
}
