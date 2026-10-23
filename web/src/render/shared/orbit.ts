/**
 * The game's orbit camera controls: dragging turns and pans the camera, the wheel and the keys
 * zoom, turn and pan it, all with inertia, within its limits. Engine-neutral: it owns the pose
 * (alpha, beta, radius, target, in game space) and the renderer copies it to its camera each frame.
 *
 * A port of Babylon.js 9.30's `ArcRotateCamera` input and motion (`arcRotateCamera.ts`
 * `_checkInputs` / `_applyRotationAndZoomDelta` / `_applyPanDelta` / `_checkLimits`,
 * `cameraMovement.ts`, `arcRotateCameraMovement.ts` and the pointer, mouse wheel and keyboard
 * inputs), so the camera moves exactly as it did: the same numbers in the same order. Babylon.js is
 * Copyright (c) Microsoft Corporation, under the Apache License 2.0 (THIRD-PARTY-LICENSES.txt).
 * Left out: touch (pinch, two-finger pan; the game is desktop only), pointer lock, restoring a
 * stored state, interpolation, alpha limits and the target's height limit (the game sets none).
 *
 * Conventions as Babylon's left-handed orbit camera (game space): the camera stands at
 * `target + radius * (cos α sin β, cos β, sin α sin β)`, y up.
 */

/** Babylon's `Epsilon`: inertia below this stops. */
const EPSILON = 0.001;
/** `CameraMovement.referenceFrameRate`: inertia is per frame at this rate, whatever the frame rate. */
const REFERENCE_FPS = 60;
/** A wheel delta in lines, in pixels (Babylon's `FfMultiplier`, Firefox). */
const LINE_PIXELS = 40;

const KEY = { up: [38], down: [40], left: [37], right: [39], zoomIn: [187, 107], zoomOut: [189, 109] };
const ALL_KEYS = new Set([...KEY.up, ...KEY.down, ...KEY.left, ...KEY.right, ...KEY.zoomIn, ...KEY.zoomOut, 220]);

type Interaction = 'rotate' | 'pan' | 'zoom';

/** What a pointer drag does (Babylon's default input map): Ctrl+left pans, left turns, right pans; the middle button nothing. */
function pointerInteraction(button: number, ctrl: boolean): Interaction | null {
  if (button === 0) return ctrl ? 'pan' : 'rotate';
  return button === 2 ? 'pan' : null;
}

/** What a held key does: + and - zoom, Ctrl+arrows pan, Alt+arrows zoom, arrows turn. */
function keyInteraction(key: number, ctrl: boolean, alt: boolean): Interaction {
  if (KEY.zoomIn.includes(key) || KEY.zoomOut.includes(key)) return 'zoom';
  if (ctrl) return 'pan';
  return alt ? 'zoom' : 'rotate';
}

/** Babylon's `CameraMovement`: input in pixels becomes a velocity with inertia, the same at any frame rate. */
class Movement {
  rotationInertia = 0.9;
  zoomInertia = 0.9;
  panInertia = 0.9;
  activeInput = false;
  readonly rotationAccumulated = { x: 0, y: 0 };
  readonly panAccumulated = { x: 0, y: 0 };
  zoomAccumulated = 0;
  readonly rotationDelta = { x: 0, y: 0 };
  readonly panDelta = { x: 0, y: 0 };
  zoomDelta = 0;
  private readonly rotationVelocity = { x: 0, y: 0 };
  private readonly panVelocity = { x: 0, y: 0 };
  private zoomVelocity = 0;
  private prevFrameMs = 0;
  private dtMs = 0;

  /** This frame's deltas from the input since the last (`computeCurrentFrameDeltas`); `dtMs`: the frame's time (0: unknown). */
  computeDeltas(dtMs: number): void {
    this.dtMs = dtMs;
    const dt = this.effectiveMs(dtMs);
    const rv = this.rotationVelocity;
    const pv = this.panVelocity;
    const ra = this.rotationAccumulated;
    const pa = this.panAccumulated;
    if (this.zoomVelocity === 0 && this.zoomAccumulated === 0 && pv.x === 0 && pv.y === 0 && pa.x === 0 && pa.y === 0 && rv.x === 0 && rv.y === 0 && ra.x === 0 && ra.y === 0 && !this.activeInput) {
      this.panDelta.x = this.panDelta.y = this.rotationDelta.x = this.rotationDelta.y = this.zoomDelta = 0;
      if (dtMs > 0) this.prevFrameMs = dtMs;
      return;
    }
    pv.x = this.velocity(pv.x, pa.x, this.panInertia);
    pv.y = this.velocity(pv.y, pa.y, this.panInertia);
    this.panDelta.x = pv.x * dt;
    this.panDelta.y = pv.y * dt;
    rv.x = this.velocity(rv.x, ra.x, this.rotationInertia);
    rv.y = this.velocity(rv.y, ra.y, this.rotationInertia);
    this.rotationDelta.x = rv.x * dt;
    this.rotationDelta.y = rv.y * dt;
    this.zoomVelocity = this.velocity(this.zoomVelocity, this.zoomAccumulated, this.zoomInertia);
    this.zoomDelta = this.zoomVelocity * dt;
    if (dtMs > 0) this.prevFrameMs = dtMs;
    this.zoomAccumulated = 0;
    pa.x = pa.y = ra.x = ra.y = 0;
    this.activeInput = false;
  }

  resetRotation(): void {
    this.rotationVelocity.x = this.rotationVelocity.y = 0;
    this.rotationAccumulated.x = this.rotationAccumulated.y = 0;
  }

  resetZoom(): void {
    this.zoomVelocity = 0;
    this.zoomAccumulated = 0;
  }

  private effectiveMs(dtMs: number): number {
    if (dtMs > 0) return dtMs;
    if (this.prevFrameMs > 0) return this.prevFrameMs;
    return 1000 / REFERENCE_FPS;
  }

  private velocity(velocity: number, pixels: number, inertia: number): number {
    const dt = this.effectiveMs(this.dtMs);
    if (dt === 0) return velocity;
    const decay = Math.pow(inertia, dt / (1000 / REFERENCE_FPS));
    let v = velocity * decay;
    if (pixels !== 0 || this.activeInput) {
      const oneMinus = 1 - inertia;
      v += (pixels / dt) * (oneMinus > 0 ? (1 - decay) / oneMinus : 1);
    } else if (Math.abs(v) < 1e-6) {
      v = 0;
    }
    return v;
  }
}

export interface OrbitLimits {
  radius: readonly [number, number];
  beta: readonly [number, number];
}

/** The orbit camera's pose and controls. */
export class OrbitControls {
  alpha: number;
  beta: number;
  radius: number;
  readonly target = { x: 0, y: 0, z: 0 };
  limits: OrbitLimits;

  /** Turning and zooming inertia (per frame at 60 fps), and panning's. */
  get inertia(): number {
    return this.movement.rotationInertia;
  }
  set inertia(value: number) {
    this.movement.rotationInertia = this.movement.zoomInertia = value;
  }
  get panningInertia(): number {
    return this.movement.panInertia;
  }
  set panningInertia(value: number) {
    this.movement.panInertia = value;
  }
  /** Pixels of drag per radian (higher: slower). */
  angularSensibilityX = 1000;
  angularSensibilityY = 1000;
  /** Pixels of drag per metre of panning. */
  panningSensibility = 1000;
  /** The wheel zooms by this share of the radius (0: by `wheelPrecision` instead). */
  wheelDeltaPercentage = 0;
  wheelPrecision = 3;
  /** Panning stays on the ground plane (the target never rises or sinks). */
  mapPanning = false;
  /** The keys: radians a frame (at 60 fps), and their panning and zooming sensibilities. */
  keyAngularSpeed = 0.01;
  keyPanningSensibility = 50;
  keyZoomingSensibility = 25;
  /** The mouse buttons that move the camera (`setLeftDragCamera` takes the left one out). */
  buttons: readonly number[] = [0, 1, 2];
  /** Input taken (dragging, the wheel, the keys); off: the pose stays wherever it is set. */
  enabled = true;
  /** Legacy-style panning inertia (Babylon's `inertialPanningX/Y`): what is left of a pan, applied and decayed each frame. */
  inertialPanningX = 0;
  inertialPanningY = 0;

  private readonly movement = new Movement();
  /** The pose the last frame was drawn with (`pan`). */
  private viewAlpha = 0;
  private viewBeta = 0;
  private inertialAlpha = 0;
  private inertialBeta = 0;
  private inertialRadius = 0;
  private readonly keys: number[] = [];
  private ctrlKey = false;
  private altKey = false;
  /** The drag in progress: its pointer, where it was last, and what it does. */
  private drag: { id: number; x: number; y: number; interaction: Interaction | null } | null = null;

  constructor(pose: { alpha: number; beta: number; radius: number }, limits: OrbitLimits) {
    this.alpha = pose.alpha;
    this.beta = pose.beta;
    this.radius = pose.radius;
    this.limits = limits;
  }

  /**
   * Turning, zooming and panning that are left over (Babylon's `inertialAlphaOffset` & co.):
   * setting one to 0 also stops that part of the input's motion, as Babylon's setters do (panning's
   * doesn't: `inertialPanningX/Y` are plain fields there too).
   */
  get inertialAlphaOffset(): number {
    return this.inertialAlpha !== 0 ? this.inertialAlpha : this.movement.rotationAccumulated.x !== 0 ? this.movement.rotationAccumulated.x : small(this.movement.rotationDelta.x);
  }
  set inertialAlphaOffset(value: number) {
    this.inertialAlpha = value;
    if (value === 0) this.movement.resetRotation();
  }
  get inertialBetaOffset(): number {
    return this.inertialBeta !== 0 ? this.inertialBeta : this.movement.rotationAccumulated.y !== 0 ? this.movement.rotationAccumulated.y : small(this.movement.rotationDelta.y);
  }
  set inertialBetaOffset(value: number) {
    this.inertialBeta = value;
    if (value === 0) this.movement.resetRotation();
  }
  get inertialRadiusOffset(): number {
    return this.inertialRadius !== 0 ? this.inertialRadius : this.movement.zoomAccumulated !== 0 ? this.movement.zoomAccumulated : small(this.movement.zoomDelta);
  }
  set inertialRadiusOffset(value: number) {
    this.inertialRadius = value;
    if (value === 0) this.movement.resetZoom();
  }

  /** Stops what is left of the player's last turn, zoom and pan (as zeroing Babylon's inertial offsets does). */
  stopInertia(): void {
    this.inertialAlphaOffset = this.inertialBetaOffset = this.inertialRadiusOffset = 0;
    this.inertialPanningX = this.inertialPanningY = 0;
  }

  /**
   * Listens to `element` (the canvas) for dragging, the wheel and the keys, in the bubble phase (the
   * game's own input, in the capture phase, sees them first). Returns what stops listening.
   */
  attach(element: HTMLElement): () => void {
    const down = (e: PointerEvent) => {
      if (!this.enabled || !this.buttons.includes(e.button) || this.drag) return;
      try {
        element.setPointerCapture(e.pointerId);
      } catch {
        // (A pointer that is already gone.)
      }
      this.drag = { id: e.pointerId, x: e.clientX, y: e.clientY, interaction: pointerInteraction(e.button, e.ctrlKey) };
    };
    const move = (e: PointerEvent) => {
      const d = this.drag;
      if (!d || d.id !== e.pointerId) return;
      const dx = e.clientX - d.x;
      const dy = e.clientY - d.y;
      d.x = e.clientX;
      d.y = e.clientY;
      if (!this.enabled) return;
      const m = this.movement;
      if (d.interaction === 'pan') {
        const scale = this.panningSensibility !== 0 ? 1 / this.panningSensibility : 0;
        if (scale !== 0) {
          m.activeInput = true;
          m.panAccumulated.x += -dx * scale;
          m.panAccumulated.y += dy * scale;
        }
      } else if (d.interaction === 'rotate') {
        m.activeInput = true;
        m.rotationAccumulated.x += -dx / this.angularSensibilityX;
        m.rotationAccumulated.y += -dy / this.angularSensibilityY;
      }
    };
    const up = (e: PointerEvent) => {
      if (!this.drag || this.drag.id !== e.pointerId) return;
      try {
        element.releasePointerCapture(e.pointerId);
      } catch {
        // (Released already.)
      }
      this.drag = null;
    };
    const wheel = (e: WheelEvent) => {
      if (this.enabled) this.wheel(e);
    };
    const keydown = (e: KeyboardEvent) => this.key(e, true);
    const keyup = (e: KeyboardEvent) => this.key(e, false);
    const blur = () => {
      this.keys.length = 0;
      this.drag = null;
    };
    // (Babylon's pointer input keeps the browser's menu off the canvas: a right-drag pans.)
    const menu = (e: Event) => e.preventDefault();
    element.addEventListener('pointerdown', down);
    element.addEventListener('pointermove', move);
    element.addEventListener('pointerup', up);
    element.addEventListener('pointercancel', up);
    element.addEventListener('lostpointercapture', up);
    element.addEventListener('wheel', wheel, { passive: true });
    element.addEventListener('keydown', keydown);
    element.addEventListener('keyup', keyup);
    element.addEventListener('blur', blur);
    element.addEventListener('contextmenu', menu);
    window.addEventListener('blur', blur);
    return () => {
      element.removeEventListener('pointerdown', down);
      element.removeEventListener('pointermove', move);
      element.removeEventListener('pointerup', up);
      element.removeEventListener('pointercancel', up);
      element.removeEventListener('lostpointercapture', up);
      element.removeEventListener('wheel', wheel);
      element.removeEventListener('keydown', keydown);
      element.removeEventListener('keyup', keyup);
      element.removeEventListener('blur', blur);
      element.removeEventListener('contextmenu', menu);
      window.removeEventListener('blur', blur);
      blur();
    };
  }

  /**
   * One frame: the held keys' input, then the input's motion, then what is left of earlier
   * motion, then the limits. `dtMs`: the frame's time (0: unknown, as the frame before).
   * `keyScale` scales the keys' nudges (1 at 60 fps; the renderer keeps them as fast at any rate).
   */
  step(dtMs: number, keyScale = 1): void {
    if (this.enabled) this.checkKeys(keyScale);
    // Panning moves along the view as it was drawn last (Babylon's last view matrix), before this frame's turn.
    this.viewAlpha = this.alpha;
    this.viewBeta = this.beta;
    const m = this.movement;
    m.computeDeltas(dtMs);
    if (m.rotationDelta.x !== 0 || m.rotationDelta.y !== 0 || m.zoomDelta !== 0) this.rotateAndZoom(m.rotationDelta.x, m.rotationDelta.y, m.zoomDelta);
    if (m.panDelta.x !== 0 || m.panDelta.y !== 0) this.pan(m.panDelta.x, m.panDelta.y);
    if (this.inertialAlpha !== 0 || this.inertialBeta !== 0 || this.inertialRadius !== 0) {
      this.rotateAndZoom(this.inertialAlpha, this.inertialBeta, this.inertialRadius);
      const k = this.inertia;
      this.inertialAlpha *= k;
      this.inertialBeta *= k;
      this.inertialRadius *= k;
      if (Math.abs(this.inertialAlpha) < EPSILON) this.inertialAlpha = 0;
      if (Math.abs(this.inertialBeta) < EPSILON) this.inertialBeta = 0;
      if (Math.abs(this.inertialRadius) < EPSILON) this.inertialRadius = 0;
    }
    if (this.inertialPanningX !== 0 || this.inertialPanningY !== 0) {
      this.pan(this.inertialPanningX, this.inertialPanningY);
      this.inertialPanningX *= this.panningInertia;
      this.inertialPanningY *= this.panningInertia;
      if (Math.abs(this.inertialPanningX) < EPSILON) this.inertialPanningX = 0;
      if (Math.abs(this.inertialPanningY) < EPSILON) this.inertialPanningY = 0;
    }
    this.checkLimits();
  }

  /** Keeps beta and the radius within the limits (a radius held at one stops zooming, as Babylon's `_checkLimits`). */
  checkLimits(): void {
    const { beta, radius } = this.limits;
    if (this.beta < beta[0]) this.beta = beta[0];
    if (this.beta > beta[1]) this.beta = beta[1];
    if (this.radius < radius[0]) {
      this.radius = radius[0];
      this.inertialRadiusOffset = 0;
    }
    if (this.radius > radius[1]) {
      this.radius = radius[1];
      this.inertialRadiusOffset = 0;
    }
  }

  private rotateAndZoom(alpha: number, beta: number, radius: number): void {
    // (Left-handed: Babylon's handedness multiplier is 1.)
    this.alpha += this.beta < 0 ? -alpha : alpha;
    this.beta += beta;
    this.radius -= radius;
  }

  /** Moves the target by (x, y) in the camera's own axes (right, up); on the ground with `mapPanning`. */
  private pan(x: number, y: number): void {
    const sa = Math.sin(this.viewAlpha);
    const ca = Math.cos(this.viewAlpha);
    const sb = Math.sin(this.viewBeta);
    const cb = Math.cos(this.viewBeta);
    // The camera's forward (target - eye), right (up × forward, normalized) and up (forward × right), as Babylon's LookAtLH.
    const fx = -ca * sb;
    const fy = -cb;
    const fz = -sa * sb;
    let rx = fz;
    let rz = -fx;
    const rl = Math.hypot(rx, rz) || 1;
    rx /= rl;
    rz /= rl;
    const ux = fy * rz;
    const uy = fz * rx - fx * rz;
    const uz = -fy * rx;
    // (`panningAxis` (1, 1, 0): no movement along the view.)
    const dx = x * rx + y * ux;
    let dy = y * uy;
    const dz = x * rz + y * uz;
    // `mapPanning`: up × ((d × up)) keeps the horizontal part; without it (and with a panning axis that has y) d stays as it is.
    if (this.mapPanning) dy = 0;
    this.target.x += dx;
    this.target.y += dy;
    this.target.z += dz;
  }

  private wheel(e: WheelEvent): void {
    const wheelDelta = -(e.deltaY * (e.deltaMode === WheelEvent.DOM_DELTA_LINE ? LINE_PIXELS : 1));
    let delta: number;
    if (this.wheelDeltaPercentage) {
      delta = this.wheelZoom(wheelDelta, this.radius);
      if (delta > 0) {
        // Zooming in: as far as the radius will have come by the time this zoom's inertia runs out.
        let estimated = this.radius;
        let inertia = this.inertialRadiusOffset + delta;
        for (let i = 0; i < 20; i++) {
          if (estimated <= inertia) break;
          if (Math.abs(inertia * this.inertia) < 0.001) break;
          estimated -= inertia;
          inertia *= this.inertia;
        }
        delta = this.wheelZoom(wheelDelta, Math.max(0, estimated));
      }
    } else {
      delta = wheelDelta / (this.wheelPrecision * 40);
    }
    if (delta) this.movement.zoomAccumulated += delta;
  }

  /** Babylon's `_computeDeltaFromMouseWheelLegacyEvent`: a share of the radius, a little less going in than out. */
  private wheelZoom(wheelDelta: number, radius: number): number {
    const d = wheelDelta * 0.01 * this.wheelDeltaPercentage * radius;
    return wheelDelta > 0 ? d / (1 + this.wheelDeltaPercentage) : d * (1 + this.wheelDeltaPercentage);
  }

  private key(e: KeyboardEvent, down: boolean): void {
    if (e.metaKey) return;
    // Every key press updates the modifiers (Ctrl pressed while an arrow is held turns it into panning).
    if (down) {
      this.ctrlKey = e.ctrlKey;
      this.altKey = e.altKey;
    }
    if (!ALL_KEYS.has(e.keyCode)) return;
    if (down) {
      if (!this.keys.includes(e.keyCode)) this.keys.push(e.keyCode);
    } else {
      const i = this.keys.indexOf(e.keyCode);
      if (i >= 0) this.keys.splice(i, 1);
    }
  }

  /** The held keys' input this frame (Babylon's keyboard input `checkInputs`). */
  private checkKeys(scale: number): void {
    if (!this.keys.length) return;
    const m = this.movement;
    let rx = 0;
    let ry = 0;
    let px = 0;
    let py = 0;
    let rotateSpeed = 0;
    let panSpeed = 0;
    for (const key of this.keys) {
      if (key === 220) continue;
      const interaction = keyInteraction(key, this.ctrlKey, this.altKey);
      if (interaction === 'pan') {
        panSpeed = Math.max(panSpeed, 1 / (this.keyPanningSensibility / scale));
        if (KEY.left.includes(key)) px -= 1;
        else if (KEY.right.includes(key)) px += 1;
        else if (KEY.up.includes(key)) py += 1;
        else if (KEY.down.includes(key)) py -= 1;
      } else if (interaction === 'zoom') {
        const speed = 1 / (this.keyZoomingSensibility / scale);
        if (KEY.up.includes(key) || KEY.zoomIn.includes(key)) m.zoomAccumulated += speed;
        else if (KEY.down.includes(key) || KEY.zoomOut.includes(key)) m.zoomAccumulated -= speed;
      } else {
        rotateSpeed = Math.max(rotateSpeed, this.keyAngularSpeed * scale);
        if (KEY.left.includes(key)) rx -= 1;
        else if (KEY.right.includes(key)) rx += 1;
        else if (KEY.up.includes(key)) ry -= 1;
        else if (KEY.down.includes(key)) ry += 1;
      }
    }
    if (rx !== 0 || ry !== 0) {
      const l = Math.hypot(rx, ry);
      m.rotationAccumulated.x += (rx / l) * rotateSpeed;
      m.rotationAccumulated.y += (ry / l) * rotateSpeed;
    }
    if (px !== 0 || py !== 0) {
      const l = Math.hypot(px, py);
      m.panAccumulated.x += (px / l) * panSpeed;
      m.panAccumulated.y += (py / l) * panSpeed;
    }
  }
}

/** Babylon's inertial-offset getters round a current-frame delta under `Epsilon` to 0. */
function small(delta: number): number {
  return Math.abs(delta) < EPSILON ? 0 : delta;
}
