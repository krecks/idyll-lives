/** A tool in hand on the town (step 1's land brushes, step 2's editor tools: `TownStage.svelte`): the left button is its, at screen points (CSS pixels). */
export interface StageTool {
  /** Pressed at (`x`, `y`): true if it took the press (a stroke began). */
  down(x: number, y: number): boolean;
  move(x: number, y: number): void;
  up(): void;
  /** The pointer over the town with no button down (the cursor follows it). */
  hover(x: number, y: number): void;
  leave(): void;
  /** The stroke under way dropped (right-click, Escape). */
  cancel(): void;
}
