/**
 * Pointer handling on the 3D canvas. Separates clicks from camera drags and
 * throttles hover to once per animation frame. Game logic lives in the handlers.
 *
 * A handler can take a left-button gesture for itself (`press` returns true), e.g. drawing a
 * wall: moves then go to `hover`, the release to `release`, and a right-click while it is held
 * to `cancel`. The press is seen before the camera's own pointer handling (capture phase), so
 * the handler can stop the camera from turning for that drag.
 */

export interface PointerHandlers {
  click(x: number, y: number): void;
  hover(x: number, y: number): void;
  leave(): void;
  /** Left button pressed. Return true to take the gesture (no click, no camera turn). */
  press?(x: number, y: number): boolean;
  /** A taken gesture ended; `moved`: the pointer travelled past the click threshold. */
  release?(x: number, y: number, moved: boolean): void;
  /** Right-click (or right button during a taken gesture). Return true if it cancelled something. */
  cancel?(): boolean;
  /** A taken gesture ended without its release (the pointer was cancelled or the window lost focus): undo it. `cancel` when not given. */
  abort?(): void;
}

/** Movement (CSS px) after which a press counts as a camera drag, not a click. */
const DRAG_THRESHOLD = 6;

export class PointerInput {
  private downX = 0;
  private downY = 0;
  private dragging = false;
  /** A left-button gesture taken by `press`, until the button goes up. */
  private taken = false;
  /** The taken gesture was cancelled: ignore everything until the button goes up. */
  private dropped = false;
  private hoverQueued = false;
  private hoverX = 0;
  private hoverY = 0;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly handlers: PointerHandlers,
  ) {
    // Capture phase: runs before the camera's listeners on the same canvas.
    canvas.addEventListener('pointerdown', this.onDown, true);
    canvas.addEventListener('pointerup', this.onUp);
    canvas.addEventListener('pointermove', this.onMove);
    canvas.addEventListener('pointerleave', this.onLeave);
    canvas.addEventListener('contextmenu', this.onContextMenu);
    canvas.addEventListener('pointercancel', this.onLost);
    canvas.addEventListener('lostpointercapture', this.onLost);
    window.addEventListener('blur', this.onLost);
  }

  dispose(): void {
    this.canvas.removeEventListener('pointerdown', this.onDown, true);
    this.canvas.removeEventListener('pointerup', this.onUp);
    this.canvas.removeEventListener('pointermove', this.onMove);
    this.canvas.removeEventListener('pointerleave', this.onLeave);
    this.canvas.removeEventListener('contextmenu', this.onContextMenu);
    this.canvas.removeEventListener('pointercancel', this.onLost);
    this.canvas.removeEventListener('lostpointercapture', this.onLost);
    window.removeEventListener('blur', this.onLost);
  }

  private readonly onDown = (e: PointerEvent) => {
    if (e.button === 2) {
      // Right-click cancels a drawing in progress (right-drag still pans the camera).
      this.cancelTaken();
      return;
    }
    if (e.button !== 0) return;
    this.downX = e.offsetX;
    this.downY = e.offsetY;
    this.dragging = false;
    this.dropped = false;
    this.taken = this.handlers.press?.(e.offsetX, e.offsetY) ?? false;
    if (this.taken) {
      try {
        this.canvas.setPointerCapture(e.pointerId);
      } catch {
        // Synthetic events have no active pointer to capture.
      }
    }
  };

  private readonly onUp = (e: PointerEvent) => {
    if (e.button !== 0) return;
    if (this.taken || this.dropped) {
      const moved = this.dragging;
      const taken = this.taken;
      this.taken = this.dropped = false;
      this.dragging = false;
      if (taken) this.handlers.release?.(e.offsetX, e.offsetY, moved);
      return;
    }
    if (!this.dragging) this.handlers.click(e.offsetX, e.offsetY);
  };

  private readonly onMove = (e: PointerEvent) => {
    // A second button pressed during a drag arrives as a move with `button` set (chorded).
    if (e.button === 2 && this.taken) this.cancelTaken();
    if (e.buttons && Math.hypot(e.offsetX - this.downX, e.offsetY - this.downY) > DRAG_THRESHOLD) this.dragging = true;
    this.hoverX = e.offsetX;
    this.hoverY = e.offsetY;
    if (!this.hoverQueued) {
      this.hoverQueued = true;
      requestAnimationFrame(() => {
        this.hoverQueued = false;
        this.handlers.hover(this.hoverX, this.hoverY);
      });
    }
  };

  private cancelTaken(): void {
    const cancelled = this.handlers.cancel?.() ?? false;
    if (this.taken || cancelled) {
      this.dropped = this.taken;
      this.taken = false;
    }
  }

  /**
   * The release won't come: the browser took the pointer (a pen or touch gesture), or the button
   * went up in another window. A taken gesture is undone rather than left following the pointer.
   * (After a normal release the capture is already gone and nothing is taken.)
   */
  private readonly onLost = () => {
    if (!this.taken) return;
    this.taken = false;
    this.dragging = false;
    // Should the release still arrive, it's ignored.
    this.dropped = true;
    if (this.handlers.abort) this.handlers.abort();
    else this.handlers.cancel?.();
  };

  private readonly onLeave = () => {
    // While a gesture is taken the pointer is captured; leaving doesn't end it.
    if (!this.taken) this.handlers.leave();
  };
  private readonly onContextMenu = (e: Event) => {
    e.preventDefault();
    if (this.taken) this.cancelTaken();
  };
}

/** What `routeKey` reads of a key press. */
export interface KeyPress {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

/** What is open over the game when a key comes (`routeKey`). */
export interface KeyContext {
  /** The pause menu is open. */
  paused: boolean;
  /** A dialog is over the game (the planner, the job board, a settings or load window). */
  dialog: boolean;
  /** Focus is on the page or the canvas, not on a control: Tab is the game's then. */
  onScene: boolean;
}

/**
 * What the game does with a key press: `escape` (step back, or close the pause menu), `undo` and
 * `redo`, `game` (its shortcuts), or `none` (the key is the browser's or the dialog's).
 *
 * Behind a dialog or the pause menu the world's keys rest (Space would run time behind the
 * planner, Tab change who the job board's Join is for); only the debug keys (F3, F8) work there.
 * With Ctrl, ⌘ or Alt held a key is the browser's (⌘V pastes, Ctrl+P prints), but for undo and
 * redo. Tab moves focus as usual unless it's on the page or the canvas.
 */
export function routeKey(e: KeyPress, c: KeyContext): 'escape' | 'undo' | 'redo' | 'game' | 'none' {
  if (e.key === 'Escape') return c.dialog ? 'none' : 'escape';
  if (c.paused || c.dialog) return e.key === 'F3' || e.key === 'F8' ? 'game' : 'none';
  const command = e.ctrlKey || e.metaKey;
  if (command && !e.altKey && (e.key === 'z' || e.key === 'Z')) return e.shiftKey ? 'redo' : 'undo';
  if (e.ctrlKey && !e.altKey && !e.metaKey && (e.key === 'y' || e.key === 'Y')) return 'redo';
  if (command || e.altKey) return 'none';
  if (e.key === 'Tab' && !c.onScene) return 'none';
  return 'game';
}
