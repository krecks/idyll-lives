<script lang="ts">
  import { onMount } from 'svelte';
  import { host } from '../../game/host';
  import type { GameSession } from '../../game/session';
  import { UnloadableSave } from '../../persistence/saves';
  import { settings, systemMotion } from '../../settings/settings.svelte';
  import { app } from '../app.svelte';
  import Hud from '../Hud.svelte';
  import { game } from '../state.svelte';
  import LoadingScreen from './LoadingScreen.svelte';
  import PauseMenu from './PauseMenu.svelte';
  import Planner from '../planner/Planner.svelte';
  import Creation from '../Creation.svelte';
  import CustomiseColours from '../buy/CustomiseColours.svelte';

  /** The game screen: the canvas lives in the host (behind every screen); this adds the HUD. */
  let session = $state.raw<GameSession | null>(null);
  let error = $state('');
  let errorTitle = $state("Couldn't start the game");
  /** The game is loaded (usually already, prepared behind the menus); until then, a loading screen. */
  let ready = $state(false);
  /** The game has been revealed (the fly-in from the menu is done): the HUD shows. */
  let live = $state(false);

  onMount(() => {
    let left = false;
    const snapshot = $state.snapshot(settings);
    host
      .prepare(app.request!, snapshot, false)
      .then((s) => {
        // Mount the HUD now, while the menu's camera flies in, rather than when the game appears.
        if (!left) {
          session = s;
          ready = true;
        }
      })
      .catch(() => {});
    host
      .start(app.request!, snapshot)
      .then((s) => {
        if (!left) {
          session = s;
          live = true;
        }
      })
      .catch((err: unknown) => {
        // Left (or replaced, e.g. by a dev hot reload) before it started: nothing went wrong.
        if (left) return;
        console.error(err);
        error = err instanceof Error ? err.message : String(err);
        if (err instanceof UnloadableSave) errorTitle = "Couldn't open this save";
      });
    return () => {
      left = true;
      host.release();
    };
  });

  // Live settings (effects, camera, free will, autosave; the system's reduce motion too).
  $effect(() => {
    const s = $state.snapshot(settings);
    void systemMotion.reduced;
    session?.applySettings(s);
  });

  /** Another window has focus. */
  let away = $state(false);

  // The world stands still while a menu (or the crash screen) is open (and, if the player wants, while they're in another window).
  $effect(() => {
    session?.setMenuPause(game.pauseMenu || app.overlay !== null || game.plannerOpen || app.crash !== null || (settings.pauseOnBlur && away));
  });

  function interfaceFailed(error: unknown) {
    console.error('[ui] the game interface failed', error);
    queueMicrotask(() => app.crashed({ kind: 'interface', detail: error instanceof Error ? error.message : String(error) }));
  }
</script>

<svelte:window onblur={() => (away = true)} onfocus={() => (away = false)} />

{#if session}
  <!-- An error in the interface: the crash screen, while the game (still running here) can be saved. -->
  <svelte:boundary onerror={interfaceFailed}>
    <div class="scaled hud-root" class:prelive={!live}><Hud /></div>
    {#if app.loading}
      <LoadingScreen progress={app.loading} />
    {/if}
    {#if game.plannerOpen && game.mode === 'live'}<Planner />{/if}
    {#if game.mode === 'create'}<Creation />{/if}
    {#if game.customising}<CustomiseColours def={game.customising.def} objectId={game.customising.objectId} />{/if}
    {#if game.pauseMenu}<PauseMenu {session} />{/if}
    {#snippet failed()}{/snippet}
  </svelte:boundary>
{:else if error}
  <div class="splash">
    <div class="glass box">
      <b>{errorTitle}</b>
      <span class="error">{error}</span>
      <button class="btn" onclick={() => app.toMenu()}>Back to menu</button>
    </div>
  </div>
{:else if !ready}
  <LoadingScreen progress={app.loading ?? { label: 'Moving in', value: 0 }} />
{/if}

<style>
  .hud-root {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  /* Built behind the menu's fly-in; shown when the game is revealed. */
  .hud-root.prelive {
    visibility: hidden;
  }
  /* An error starting the game: a small card over the scene. */
  .splash {
    position: absolute;
    inset: auto 0 48px 0;
    display: grid;
    place-items: center;
    pointer-events: none;
  }
  .box {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 22px;
    max-width: 480px;
    text-align: center;
    font-weight: 600;
    border-radius: var(--radius-pill);
    pointer-events: auto;
    animation: pill 420ms var(--ease) both;
  }
  .box:has(.error) {
    flex-direction: column;
    border-radius: var(--radius-lg);
    padding: 24px 32px;
  }
  @keyframes pill {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }
  .error {
    color: var(--bad);
    font-weight: 500;
  }
</style>
