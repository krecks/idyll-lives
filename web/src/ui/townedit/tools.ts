/**
 * Step 2's town editor tools (Select, Street, Lot, Village, Remove): what each one registers so the
 * editor can offer it. A tool has an id, a label and an icon for the palette, gives the town stage
 * the pointer while it is in hand (`StageTool`), and may show a card beside the town (what can be
 * done to what it picked, what it is about to place). One tool is in hand at a time.
 *
 * Each tool lives in its own file here and adds one line to `TOWN_TOOLS`.
 */

import type { Component } from 'svelte';
import type { Content } from '../../content/content';
import type { Edited } from '../../game/edit/edited';
import type { NeighbourhoodDraft, Templates } from '../../game/town';
import type { TownGhosts } from '../../render/types';
import type { StageTool } from '../kit/stageTool';
import { lotTool } from './lotTool';
import type { Picked, PickWhat } from './pick';
import { removeTool } from './removeTool';
import { selectTool } from './selectTool';
import { streetTool } from './streetTool';
import { villageTool } from './villageTool';

/**
 * What the editor gives a tool and a card (`editor.svelte.ts` `TownEditorState`). Everything works
 * on the plan draft (`draft`), the town as the edits left it: the town on screen may lag it until
 * edits pause and it is built again, so what is under the pointer is found on the plan (`pick`,
 * `ground`), never by the town on screen's indices.
 */
export interface TownEditor {
  /** The town's plan draft as it stands. */
  readonly draft: NeighbourhoodDraft;
  /** The house templates (`town.ts`) and the game's content, for the edits (`game/edit/`). */
  readonly t: Templates;
  readonly content: Content;
  /** Bumped whenever the plan, what is picked or the town on screen changes (a card reads it to follow along). */
  readonly version: number;
  /** Makes an edit: a draft taken on (true), or a refusal shown with its reason (false). */
  apply(edited: Edited): boolean;
  /** A refusal shown: under the tools for a while, and as a toast. Written as the edits write them ("it bends tighter than 9 m"). */
  refuse(why: string): void;
  /** A fresh seed for an edit's dice (a new house, a name): the same seed, the same result. */
  seed(): number;
  /** The point on the ground under a screen point (CSS pixels, as the stage gives them), if the town is there. */
  ground(x: number, y: number): { x: number; z: number } | null;
  /** What is under a screen point on the plan, of what `what` allows (all: a lot, a building, a road, a village or the main town). */
  pick(x: number, y: number, what?: PickWhat): Picked | null;
  /** What is picked (its card shows), and picking something (null: nothing). */
  readonly selected: Picked | null;
  select(p: Picked | null): void;
  /** What the pointer is over, lit on the town (null: nothing). */
  hover(p: Picked | null): void;
  /** The tool's own ghosts over the town (a lot about to be placed, what a removal takes), drawn over the edits'; null: none. */
  preview(g: Partial<TownGhosts> | null): void;
  /** A line under the tools about what the tool would do where the pointer is (why it can't, say); null: none. */
  hint(text: string | null): void;
  /** The town's name, as step 2's own name field has it (the two kept as one). */
  townName: string;
  /** The tool in hand (its `TownTool.id`), and a tool taken in hand (the same one again puts it down; null: none), as a card's action may ("Move" a village). */
  readonly toolId: string | null;
  take(id: string | null): void;
}

/**
 * What the town stage does with the pointer while a tool is in hand (`StageTool`), and a key pressed
 * meanwhile (not in a text field): true if the tool took it (the editor then does nothing else with
 * it: Esc, say, drops what the tool is drawing rather than putting the tool down).
 */
export interface TownStageTool extends StageTool {
  key?(e: KeyboardEvent): boolean;
}

export interface TownTool {
  id: string;
  label: string;
  /** Its icon's asset key (`Icon.svelte`). */
  icon: string;
  /** What it does, in a line (the palette's title). */
  does: string;
  /** What the town stage does with the pointer while the tool is in hand. */
  stage(editor: TownEditor): TownStageTool;
  /** The tool's card, shown beside the town while the tool is in hand. */
  card?: Component<{ editor: TownEditor }>;
}

/** The tools, in the order the palette shows them. */
export const TOWN_TOOLS: readonly TownTool[] = [selectTool, streetTool, lotTool, villageTool, removeTool];
