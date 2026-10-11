/**
 * The models to choose from: the built-in ones and those the player added from Hugging Face
 * (kept in this browser). Adding only describes a model (`hub.ts`); it downloads when first used.
 */

import { describeHubModel } from './hub';
import { BUILT_IN, DEFAULT_MODEL, type ModelSpec } from './models';

const KEY = 'littlelives.llmModels';

export const library = $state({ added: read() });

function read(): ModelSpec[] {
  try {
    const list = JSON.parse(localStorage.getItem(KEY) ?? '[]') as ModelSpec[];
    return Array.isArray(list) ? list.filter((m) => m && typeof m.id === 'string' && m.added) : [];
  } catch {
    return [];
  }
}

function save(): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(library.added));
  } catch {
    // Storage unavailable: added models last this session.
  }
}

/** Every model, built-in first. */
export function allModels(): ModelSpec[] {
  return [...BUILT_IN, ...library.added];
}

/** The model with this id, or the default. */
export function modelById(id: string | null | undefined): ModelSpec {
  return allModels().find((m) => m.id === id) ?? DEFAULT_MODEL;
}

/** Looks a Hugging Face repository up and adds its model (or updates it); throws with a reason. */
export async function addModel(repo: string): Promise<ModelSpec> {
  const spec = await describeHubModel(repo);
  library.added = [...library.added.filter((m) => m.id !== spec.id), spec];
  save();
  return spec;
}

export function removeModel(id: string): void {
  library.added = library.added.filter((m) => m.id !== id);
  save();
}
