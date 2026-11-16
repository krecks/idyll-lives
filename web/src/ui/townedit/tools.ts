/**
 * Step 2's town editor tools (Select, Street, Lot, Village, Remove): what each one registers so the
 * editor can offer it. A tool has an id, a label and an icon for the palette, gives the town stage
 * the pointer while it is in hand (`StageTool`), and may show a card beside the town (what can be
 * done to what it picked, what it is about to place). One tool is in hand at a time.
 *
 * Each tool lives in its own file here and adds one line to `TOWN_TOOLS`.
 */

import type { Component } from 'svelte';
import type { Edited } from '../../game/edit/edited';
import type { NeighbourhoodDraft } from '../../game/town';
import type { StageTool } from '../kit/stageTool';

/** What the editor gives a tool: the town's plan draft as it stands, and a way to make an edit to it. */
export interface TownEditor {
  readonly draft: NeighbourhoodDraft;
  /** Makes an edit (`game/edit/`): a draft taken on, a refusal shown with its reason. */
  apply(edited: Edited): void;
}

export interface TownTool {
  id: string;
  label: string;
  /** Its icon's asset key (`Icon.svelte`). */
  icon: string;
  /** What the town stage does with the pointer while the tool is in hand. */
  stage(editor: TownEditor): StageTool;
  /** The tool's card, shown beside the town while the tool is in hand. */
  card?: Component<{ editor: TownEditor }>;
}

/** The tools, in the order the palette shows them. */
export const TOWN_TOOLS: readonly TownTool[] = [];
