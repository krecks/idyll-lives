/**
 * One audio context for the whole game, with a bus each for interface sounds (`ui/sfx.ts`),
 * resident voices (`voice/player.ts`) and the sounds of the world (`audio/world.ts`), each with
 * its own volume setting, all through a limiter and the master volume.
 *
 * The mix: while a resident speaks the world's sounds give way to the voice (`DUCK`), and what the
 * buses add up to goes through a limiter (`LIMIT`), so a shower, a blender and two voices at once
 * are held under full scale and not clipped. Below its threshold the limiter changes nothing.
 *
 * The context starts with the player's first click or key press (browsers allow sound only after
 * one); a context the browser stopped (Safari's `interrupted`) starts again with the next one.
 * It stops while the tab is hidden, and with "Mute in background" the master volume fades out
 * while another window has focus. While it isn't running nothing new plays, so nothing is left
 * queued up to play all at once on return.
 */

import { settings } from '../settings/settings.svelte';

export type Bus = 'effects' | 'voices' | 'world';

/** Interface sounds are quiet by design. */
const EFFECTS_LEVEL = 0.5;
/** Seconds the master volume takes to fade out or back in as the window loses or gets focus. */
const FOCUS_FADE = 0.15;
/**
 * The world under a voice: the share of its level left while a resident speaks (about -6 dB: the line is
 * heard over a tap or a television without the room going silent), and the seconds it takes to give way
 * and to come back once the last voice has ended.
 */
export const DUCK = { level: 0.5, down: 0.08, up: 0.5 };
/**
 * The limiter over everything: what goes over `threshold` (dBFS) is held there (`ratio` to one, no knee),
 * caught within `attack` seconds and let go over `release`. The sounds are made well under it one by one
 * (`tools/audio/listen.ts`: the loudest loop peaks at -11 dBFS), so it works only when many sound at once.
 */
export const LIMIT = { threshold: -3, ratio: 20, attack: 0.003, release: 0.25 };

let ctx: AudioContext | null = null;
let master: GainNode | null = null;
/** What the buses play into: the limiter, then the master volume. */
let mix: DynamicsCompressorNode | null = null;
/** The world's bus plays through this: its level under a voice. */
let duck: GainNode | null = null;
/** Voice clips playing now. */
let speaking = 0;
const buses = new Map<Bus, GainNode>();
/** The master volume last asked for (it fades there). */
let masterTarget = -1;
/** Another window has focus. */
let away = false;
/** Called just before the context stops for a hidden tab. */
const beforeHide = new Set<() => void>();

/** The audio context while sound can play; null before the first click or key press, in a hidden tab, or without Web Audio. */
export function audioContext(): AudioContext | null {
  // Loaded after the player has already clicked (this page has had a gesture): start now.
  if (!ctx && navigator.userActivation?.hasBeenActive) start();
  return ctx?.state === 'running' && !document.hidden ? ctx : null;
}

/** The bus's input, at its current volume. */
export function bus(name: Bus): GainNode | null {
  const ac = audioContext();
  if (!ac || !master) return null;
  let gain = buses.get(name);
  if (!gain) {
    gain = ac.createGain();
    gain.connect((name === 'world' ? duck : null) ?? mix ?? master);
    buses.set(name, gain);
  }
  gain.gain.value = level(name);
  fadeMaster();
  return gain;
}

/**
 * A resident's voice starts (`on`) or ends: the world's sounds give way while any is speaking (`DUCK`).
 * Every start is to be followed by one end.
 */
export function voiceSpeaking(on: boolean): void {
  speaking = Math.max(0, speaking + (on ? 1 : -1));
  if (!ctx || !duck) return;
  duck.gain.setTargetAtTime(speaking > 0 ? DUCK.level : 1, ctx.currentTime, (speaking > 0 ? DUCK.down : DUCK.up) / 3);
}

/**
 * What the mix is doing now, to check it by (null before sound can play): how many voices are speaking, the
 * world's level under them (1: not held down) and how many dB the limiter is taking off (0: nothing).
 */
export function mixNow(): { speaking: number; duck: number; limiting: number } | null {
  return duck && mix ? { speaking, duck: duck.gain.value, limiting: -mix.reduction } : null;
}

/** `stop` runs just before sound stops for a hidden tab (sounds that wait for their end, like voices, end there). Returns a function that removes it. */
export function onHide(stop: () => void): () => void {
  beforeHide.add(stop);
  return () => beforeHide.delete(stop);
}

function level(name: Bus): number {
  switch (name) {
    case 'effects':
      return EFFECTS_LEVEL;
    case 'voices':
      return settings.voiceVolume;
    case 'world':
      return settings.worldSound ? settings.worldVolume : 0;
  }
}

/** Applies changed volume settings right away. */
export function updateVolumes(): void {
  for (const [name, gain] of buses) gain.gain.value = level(name);
  fadeMaster();
}

/** The master volume: the setting, or nothing while another window has focus (with "Mute in background"). */
function masterLevel(): number {
  const v = away && settings.muteInBackground ? 0 : settings.masterVolume;
  return Number.isFinite(v) ? Math.max(0, Math.min(1, v)) : 1;
}

function fadeMaster(): void {
  if (!ctx || !master) return;
  const target = masterLevel();
  if (target === masterTarget) return;
  masterTarget = target;
  master.gain.setTargetAtTime(target, ctx.currentTime, FOCUS_FADE / 3);
}

function start(): void {
  try {
    ctx = new AudioContext();
  } catch {
    return;
  }
  master = ctx.createGain();
  master.connect(ctx.destination);
  master.gain.value = masterTarget = masterLevel();
  // The buses' sum through the limiter, then the master volume (a quiet master is not limited any the less).
  const limiter = ctx.createDynamicsCompressor();
  limiter.threshold.value = LIMIT.threshold;
  limiter.knee.value = 0;
  limiter.ratio.value = LIMIT.ratio;
  limiter.attack.value = LIMIT.attack;
  limiter.release.value = LIMIT.release;
  limiter.connect(master);
  mix = limiter;
  duck = ctx.createGain();
  duck.gain.value = speaking > 0 ? DUCK.level : 1;
  duck.connect(mix);
}

/** A click or key press: sound may start (or start again). */
function onGesture(): void {
  away = false;
  if (!ctx) start();
  else if (ctx.state !== 'running' && ctx.state !== 'closed' && !document.hidden) ctx.resume().catch(() => {});
  fadeMaster();
}

function onVisibility(): void {
  if (!ctx || ctx.state === 'closed') return;
  if (document.hidden) {
    for (const stop of beforeHide) stop();
    ctx.suspend().catch(() => {});
  } else ctx.resume().catch(() => {});
}

function onFocus(e: FocusEvent): void {
  away = e.type === 'blur';
  fadeMaster();
}

// Pointer down for a mouse, up for touch and pens: the events browsers count as a gesture.
window.addEventListener('pointerdown', onGesture, true);
window.addEventListener('pointerup', onGesture, true);
window.addEventListener('keydown', onGesture, true);
window.addEventListener('blur', onFocus);
window.addEventListener('focus', onFocus);
document.addEventListener('visibilitychange', onVisibility);
