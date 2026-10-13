/**
 * One audio context for the whole game, with a bus each for interface sounds (`ui/sfx.ts`),
 * resident voices (`voice/player.ts`) and the sounds of the world (`audio/world.ts`), each with
 * its own volume setting, all through the master volume.
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

let ctx: AudioContext | null = null;
let master: GainNode | null = null;
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
    gain.connect(master);
    buses.set(name, gain);
  }
  gain.gain.value = level(name);
  fadeMaster();
  return gain;
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
