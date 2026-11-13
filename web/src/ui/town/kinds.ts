/**
 * The town tags' kinds (docs/design/living-town.md, Town tags), each also a filter in the tag menu, most important
 * first. Where they go on screen, and which give way when they meet, is `game/tagLayout.ts` (by `tags.ts` `tagRank`).
 */

/** What a tag is for, also its filter (the tag menu). */
export type TagKind = 'household' | 'people' | 'places' | 'sale' | 'names';

/** Most important first (the tag menu's order). */
export const TAG_ORDER: readonly TagKind[] = ['household', 'people', 'places', 'sale', 'names'];
