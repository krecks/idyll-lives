/**
 * A land brush in the player's hand, in step 1 and in Build mode's Land tab: the pointer turned into
 * strokes of a `LandEditor` (`begin` on the press, `move` once a frame while held, so a brush held
 * still keeps working, `end` on the release), each stamp's land drawn again at once and the whole
 * stroke's once it ends, the cursor following the land under the pointer (red where the brush
 * would do nothing), and a mountain's ghost while it is dragged out. Where it draws and picks is a
 * `LandSurface`: the menus' town (`menuScene`) or the game's renderer.
 */

import { heightAt, LAND_CELL, type LandPatch, type LandRect } from '../../game/land';
import type { Brush, MountainGhost } from '../../game/landBrush';
import type { LandEditor } from '../../game/landEdit';
import type { LandCursor } from '../../render/types';
import { cursorOf } from './landTools';

/** Where a brush works: picking the land, drawing it again, the cursor, projecting the ghost. */
export interface LandSurface {
  /** The land under canvas CSS pixel (`x`, `y`): a town point, or null. */
  pick(x: number, y: number): { x: number; z: number } | null;
  changed(rect: LandRect, done: boolean): void;
  cursor(cursor: LandCursor | null): void;
  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean;
}

export interface StrokeHooks {
  /** The brush in hand, read at the press (and for the cursor). */
  brush(): Brush;
  /** Before each frame's stamps: false stops the stroke there (the funds won't reach further). */
  goOn?(editor: LandEditor): boolean;
  /** A stroke ended and changed something: the patch `end` returned. */
  ended?(patch: LandPatch): void;
  /** The ghost of a mountain being dragged out, as SVG path data in screen pixels ('' for none). */
  ghost?(path: string): void;
  /** Each frame of a stroke: the stamps' time (ms) drawing them (for benches). */
  stamped?(ms: number): void;
}

/** Whether `tool` would do nothing at (`x`, `z`): built land, or water where water may not go (`LandEditor.refuses`). */
const refuses = (editor: LandEditor, brush: Brush, x: number, z: number) => editor.refuses(brush.tool, x, z);

export class LandStroke {
  /** The pointer's last canvas position, while a stroke is held. */
  private at: { x: number; y: number } | null = null;
  private frame = 0;
  private last = 0;
  private readonly out = { x: 0, y: 0 };
  /** Whether a stroke is under way. */
  get active(): boolean {
    return this.at !== null;
  }

  constructor(
    readonly editor: LandEditor,
    private readonly surface: LandSurface,
    private readonly hooks: StrokeHooks,
  ) {}

  /** The pointer at (`x`, `y`) with no button: the cursor follows it. */
  hover(x: number, y: number): void {
    if (this.at) return this.move(x, y);
    const brush = this.hooks.brush();
    const p = this.surface.pick(x, y);
    this.surface.cursor(p ? cursorOf(brush, p.x, p.z, refuses(this.editor, brush, p.x, p.z)) : null);
  }

  /** The pointer left the land's view: no cursor (a stroke under way goes on). */
  leave(): void {
    if (!this.at) this.surface.cursor(null);
  }

  /** The press: a stroke begins where the pointer is on the land (false: it isn't). */
  down(x: number, y: number): boolean {
    if (this.at) this.up();
    const p = this.surface.pick(x, y);
    if (!p) return false;
    const brush = this.hooks.brush();
    const t = performance.now();
    const rect = this.editor.begin(brush, p.x, p.z);
    if (rect) this.surface.changed(rect, false);
    this.hooks.stamped?.(performance.now() - t);
    this.surface.cursor(cursorOf(brush, p.x, p.z, refuses(this.editor, brush, p.x, p.z)));
    this.at = { x, y };
    this.last = performance.now();
    this.showGhost();
    this.frame = requestAnimationFrame(this.tick);
    return true;
  }

  /** The pointer dragged on (the stamps come with the next frame). */
  move(x: number, y: number): void {
    if (this.at) this.at = { x, y };
  }

  /** The release: the stroke ends (its patch to `ended`); null if it changed nothing. */
  up(): LandPatch | null {
    if (!this.at) return null;
    // (What the pointer did since the last frame, first.)
    this.step(performance.now(), false);
    this.stop();
    const patch = this.editor.end();
    this.hooks.ghost?.('');
    if (patch) {
      this.surface.changed(patch.rect, true);
      this.hooks.ended?.(patch);
    }
    return patch;
  }

  /** The stroke dropped (right-click, Escape, the pointer lost): taken back as if it hadn't been. */
  cancel(): void {
    if (!this.at) return;
    this.stop();
    this.hooks.ghost?.('');
    const patch = this.editor.end();
    if (!patch) return;
    const back = this.editor.undo();
    this.editor.dropRedo();
    if (back) this.surface.changed(back.rect, true);
  }

  /** Hides the cursor and drops a stroke under way (the brush put down). */
  dispose(): void {
    this.cancel();
    this.surface.cursor(null);
  }

  private stop(): void {
    cancelAnimationFrame(this.frame);
    this.at = null;
  }

  private readonly tick = (now: number) => {
    if (!this.at) return;
    // (Stopped short, as if let go: the funds won't reach further.)
    if (this.hooks.goOn && !this.hooks.goOn(this.editor)) {
      this.up();
      return;
    }
    this.step(now);
    if (this.at) this.frame = requestAnimationFrame(this.tick);
  };

  /** One frame of the stroke: stamps up to where the pointer is now (or again where it rests). */
  private step(now: number, ghost = true): void {
    const at = this.at;
    if (!at) return;
    const dt = Math.min(0.1, Math.max(0, (now - this.last) / 1000));
    this.last = now;
    const p = this.surface.pick(at.x, at.y);
    if (!p) return;
    const brush = this.editor.current?.brush ?? this.hooks.brush();
    const t = performance.now();
    const rect = this.editor.move(p.x, p.z, dt);
    if (rect) this.surface.changed(rect, false);
    this.hooks.stamped?.(performance.now() - t);
    this.surface.cursor(cursorOf(brush, p.x, p.z, refuses(this.editor, brush, p.x, p.z)));
    if (ghost) this.showGhost();
  }

  private showGhost(): void {
    if (!this.hooks.ghost) return;
    const brush = this.editor.current?.brush;
    if (!brush || (brush.tool !== 'peak' && brush.tool !== 'range')) return;
    const e = this.editor;
    const ghost = e.ghost(Math.max(LAND_CELL * 2, Math.round(brush.radius / 10)));
    const { cols, rows } = e.land;
    // (Held off the built land as the mountain will be.)
    const free = (x: number, z: number) => e.free(Math.min(cols - 1, Math.max(0, Math.round(x / LAND_CELL))), Math.min(rows - 1, Math.max(0, Math.round(z / LAND_CELL))));
    this.hooks.ghost(ghost ? ghostPath(ghost, (x, z) => heightAt(e.land, x, z), this.surface, this.out, free) : '');
  }
}

/** Lift (m) under which the ghost's lines don't go: its foot. */
const GHOST_FOOT = 0.4;

/**
 * A mountain's ghost as SVG path data (screen pixels): its surface as it would rise, lines along
 * its rows and columns over the land (`ground`), each from where it lifts off to where it comes
 * down again, about 28 of each across it. `free(x, z)`: how far the land there may rise (0 on
 * built land: the ghost lies flat there, as the mountain will).
 */
export function ghostPath(ghost: MountainGhost, ground: (x: number, z: number) => number, surface: Pick<LandSurface, 'project'>, out = { x: 0, y: 0 }, free: (x: number, z: number) => number = () => 1): string {
  const { x0, z0, step, cols, rows, lift } = ghost;
  const every = Math.max(1, Math.round(Math.max(cols, rows) / 28));
  const parts: string[] = [];
  const line = (n: number, at: (k: number) => [number, number, number]) => {
    let open = false;
    for (let k = 0; k < n; k++) {
      const [x, z, lift] = at(k);
      const up = lift * free(x, z);
      if (up < GHOST_FOOT || !surface.project(x, ground(x, z) + up, z, out)) {
        open = false;
        continue;
      }
      parts.push(`${open ? 'L' : 'M'}${out.x.toFixed(1)} ${out.y.toFixed(1)}`);
      open = true;
    }
  };
  for (let r = 0; r < rows; r += every) line(cols, (c) => [x0 + c * step, z0 + r * step, lift[r * cols + c]]);
  for (let c = 0; c < cols; c += every) line(rows, (r) => [x0 + c * step, z0 + r * step, lift[r * cols + c]]);
  return parts.join('');
}
