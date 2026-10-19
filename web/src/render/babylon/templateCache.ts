/**
 * Built model templates by key, each built once and kept (the renderer's objects and garden, the
 * street's, the town overview's, the items', the residents' props). One built without its file (a
 * stand-in box, or nothing: `ModelTemplate.failed`) is built again once the file comes:
 * `invalidate` sets it aside, so the next `get` of its key builds it afresh, and `disposeStale`
 * disposes what was set aside.
 *
 * What was set aside may still be drawn (its thin instances are the old build's), so the owner calls
 * `invalidate` only between its builds and `disposeStale` once a build after it has finished: the
 * new build draws the new template, and nothing points at the old one any more.
 */

import type { Mesh } from './core';

/** What a cache keeps: a template's meshes (and its glows'), and the files it was built without. */
export interface CachedTemplate {
  meshes: Mesh[];
  glows?: { mesh: Mesh }[];
  failed?: string[];
}

interface Entry<T> {
  promise: Promise<T>;
  /** Once built. */
  built?: { value: T };
}

export class TemplateCache<T extends CachedTemplate | null> {
  private readonly entries = new Map<string, Entry<T>>();
  /** Set aside by `invalidate`, to be disposed by `disposeStale`. */
  private stale: NonNullable<T>[] = [];

  /** The template under `key`, built by `build` the first time it's asked for (or the first time since it was set aside). */
  get(key: string, build: () => Promise<T>): Promise<T> {
    const known = this.entries.get(key);
    if (known) return known.promise;
    const entry: Entry<T> = { promise: build() };
    this.entries.set(key, entry);
    // (A build that throws stays cached as it is, as it always has; its rejection is the caller's.)
    entry.promise.then(
      (value) => (entry.built = { value }),
      () => {},
    );
    return entry.promise;
  }

  /** The template under `key` (built or being built), without building it. */
  peek(key: string): Promise<T> | undefined {
    return this.entries.get(key)?.promise;
  }

  /** The template under `key` if it's built: `undefined` while it's being built or never asked for. */
  built(key: string): T | undefined {
    return this.entries.get(key)?.built?.value;
  }

  has(key: string): boolean {
    return this.entries.has(key);
  }

  /** Forgets `key`'s template, without disposing it (whoever calls this does). */
  delete(key: string): void {
    this.entries.delete(key);
  }

  /** Forgets every template, without disposing them (whoever calls this does; what was set aside stays for `disposeStale`). */
  clear(): void {
    this.entries.clear();
  }

  /** Every template kept, by key (built or being built). */
  *[Symbol.iterator](): IterableIterator<[string, Promise<T>]> {
    for (const [key, entry] of this.entries) yield [key, entry.promise];
  }

  /** Every template built, by key. */
  *builtEntries(): IterableIterator<[string, T]> {
    for (const [key, entry] of this.entries) if (entry.built) yield [key, entry.built.value];
  }

  /**
   * The file at `url` has come: the templates built without it are set aside (still drawn where they
   * are, until `disposeStale`) and the next `get` of their keys builds them again. Their keys.
   */
  invalidate(url: string): string[] {
    const keys: string[] = [];
    for (const [key, entry] of this.entries) {
      const t = entry.built?.value;
      if (!t?.failed?.includes(url)) continue;
      this.entries.delete(key);
      this.stale.push(t);
      keys.push(key);
    }
    return keys;
  }

  /** Disposes the templates set aside (their materials are shared and stay); whether there were any. */
  disposeStale(): boolean {
    const stale = this.stale;
    this.stale = [];
    for (const t of stale) for (const mesh of [...t.meshes, ...(t.glows ?? []).map((g) => g.mesh)]) mesh.dispose(false, false);
    return stale.length > 0;
  }
}
