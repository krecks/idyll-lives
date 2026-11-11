/**
 * The dev play log's watch on the interface (`debug/devlog.ts`): the screen shown (menu, game, ...)
 * and every settings change, with what it was before.
 */

import { settings } from '../settings/settings.svelte';
import { app } from '../ui/app.svelte';
import { changes, DEVLOG, devlog, devlogPause, startDevLog } from './devlog';
import { gpuInfo } from './report';

/** Starts the dev play log with the settings it starts on, and watches the screen and the settings. */
export function watchForDevLog(): void {
  if (!DEVLOG) return;
  let last = $state.snapshot(settings) as Record<string, unknown>;
  startDevLog({ settings: last });
  void gpuInfo().then((gpu) => devlog('gpu', { gpu }));
  $effect.root(() => {
    $effect(() => {
      devlog('screen', { screen: app.screen, overlay: app.overlay });
      devlogPause();
    });
    $effect(() => {
      const now = $state.snapshot(settings) as Record<string, unknown>;
      const changed = changes(last, now);
      last = now;
      if (Object.keys(changed).length) devlog('settings', { changed });
    });
    $effect(() => {
      if (app.crash) devlog('crash', { crash: $state.snapshot(app.crash) as unknown as Record<string, unknown> });
    });
  });
}
