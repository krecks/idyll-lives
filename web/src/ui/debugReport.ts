/** Saves a debug report (F8, the button in the performance overlay, or the crash screen) and tells the player where it went. */

import { services } from './services';
import { game, toast } from './state.svelte';

/** Longest a report may take: then the player hears it failed, and F8 works again. */
const REPORT_LIMIT_MS = 25_000;

/** `make` builds and stores the report (default: the running game's, with its save while it runs). */
export async function saveDebugReport(note = '', make: (note: string) => Promise<string> = (n) => services.controls.debugReport(n)): Promise<string | null> {
  if (game.debugSaving) return null;
  game.debugSaving = true;
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    const late = new Promise<never>((_, reject) => (timer = setTimeout(() => reject(new Error('it took too long')), REPORT_LIMIT_MS)));
    const path = await Promise.race([make(note), late]);
    toast(`Debug report saved: ${path}`, 6000);
    return path;
  } catch (err) {
    console.error('[debug] report failed', err);
    toast(`Couldn't save the debug report: ${err instanceof Error ? err.message : String(err)}`, 6000);
    return null;
  } finally {
    clearTimeout(timer);
    game.debugSaving = false;
  }
}
