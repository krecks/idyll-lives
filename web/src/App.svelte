<script lang="ts">
  import { onMount, type Component } from 'svelte';
  import { host } from './game/host';
  import { menuScene } from './game/menuScene';
  import { BENCH } from './debug/benchFlags';
  import { watchLog } from './debug/log';
  import { perfEnd, perfPhase, perfStart, perfTimeAsync } from './debug/perf';
  import { fetchState, loadDebugReport } from './debug/report';
  import { loadGameData } from './game/session';
  import { latestSave, UnloadableSave } from './persistence/saves';
  import { settings } from './settings/settings.svelte';
  import { app } from './ui/app.svelte';
  import { services } from './ui/services';
  import { game, toast } from './ui/state.svelte';
  import Modal from './ui/kit/Modal.svelte';
  import ChooseHome from './ui/screens/ChooseHome.svelte';
  import CreateHousehold from './ui/screens/CreateHousehold.svelte';
  import CreateNeighbourhood from './ui/screens/CreateNeighbourhood.svelte';
  import CreateWorld from './ui/screens/CreateWorld.svelte';
  import CrashScreen from './ui/screens/CrashScreen.svelte';
  import Credits from './ui/screens/Credits.svelte';
  import LoadGame from './ui/screens/LoadGame.svelte';
  import LoadingScreen from './ui/screens/LoadingScreen.svelte';
  import MainMenu from './ui/screens/MainMenu.svelte';
  import MenuBackdrop from './ui/screens/MenuBackdrop.svelte';
  import PlayHousehold from './ui/screens/PlayHousehold.svelte';
  import SettingsPanel from './ui/screens/SettingsPanel.svelte';
  import Toasts from './ui/Toasts.svelte';

  let gameLayer: HTMLDivElement;

  /** The in-game interface is a script chunk of its own: fetched once the menu is up, or when a game starts. */
  let GameView = $state.raw<Component | null>(null);
  let gameViewLoad: Promise<unknown> | null = null;
  const loadGameView = () =>
    (gameViewLoad ??= import('./ui/screens/GameView.svelte').then(
      (m) => (GameView = m.default),
      (err: unknown) => {
        // Fetched again on the next try; a game waiting for it goes back to the menu meanwhile.
        gameViewLoad = null;
        console.error('[app] the game screen failed to load', err);
        app.newVersion = true;
        if (app.screen === 'game') app.toMenu();
      },
    ));

  /**
   * A script file that didn't load: a new version replaced this one's files (they're hashed per
   * version), or the network failed. Either way, reloading is the cure.
   */
  const MISSING_FILE = /Failed to fetch dynamically imported module|error loading dynamically imported module|Importing a module script failed|Unable to preload CSS/i;

  onMount(() => {
    host.mount(gameLayer);
    window.addEventListener('vite:preloadError', () => (app.newVersion = true));
    // Also what was caught and only logged (Babylon's shaders are fetched on first use).
    watchLog((entry) => {
      if (entry.level !== 'warn' && MISSING_FILE.test(entry.text)) app.newVersion = true;
    });
    void boot();
  });

  /** An error in the game's interface: the crash screen (outside the render that failed). */
  function interfaceFailed(error: unknown) {
    console.error('[ui] the game interface failed', error);
    queueMicrotask(() => app.crashed({ kind: 'interface', detail: error instanceof Error ? error.message : String(error) }));
  }

  /** Longest wait for one start-up step (a stuck step must not keep the menu away). */
  const STEP_LIMIT_MS = 45_000;
  const within = (p: Promise<unknown>) => Promise.race([p, new Promise((resolve) => setTimeout(resolve, STEP_LIMIT_MS))]);

  /**
   * Start-up behind the loading screen: game data, the renderer and the menu's town (its shaders
   * compiled). Then the menu, which prepares the latest save behind it so "Continue" is quick.
   */
  async function boot() {
    const booted = perfStart('boot');
    const step = (label: string, value: number) => {
      app.boot = { label, value };
      perfPhase(`boot: ${label}`);
    };
    // `?bench=<scene>`: the benchmark's settings (for this page only) before anything reads them.
    const bench = BENCH ? await import('./debug/bench') : null;
    bench?.prepareBench();
    try {
      step('Loading game data', 0.05);
      await perfTimeAsync('boot: loadGameData', loadGameData);
      app.dataReady = true;
    } catch (err) {
      console.error('[boot] the game data failed to load', err);
      app.fatal = err instanceof Error ? err.message : String(err);
      return;
    }
    try {
      const snapshot = $state.snapshot(settings);
      step('Starting the renderer', 0.15);
      await perfTimeAsync('boot: renderer', () => within(host.rendererNow(snapshot)));
      step('Building the neighbourhood', 0.3);
      await perfTimeAsync('boot: menu town', () =>
        within(
          menuScene.showcaseDraft().then((draft) => {
            menuScene.highlight({});
            menuScene.shoot({ kind: 'menu' }, 2.4);
            return menuScene.show(draft);
          }),
        ),
      );
    } catch (err) {
      console.warn('[boot]', err);
    }
    step('Ready', 1);
    perfEnd(booted);
    app.booting = false;
    void perfTimeAsync('boot: game screen chunk', loadGameView);
    await openDebugReport();
    await bench?.runBench();
  }

  // Global interface preferences.
  $effect(() => {
    const root = document.documentElement;
    root.style.setProperty('--ui-scale', String(settings.uiScale));
    root.classList.toggle('reduce-motion', settings.reducedMotion);
  });

  /** `?debugReport=<name>` (dev server): starts straight into a saved debug report's game;
   * `?state=<name>` into a state the simulation's tests wrote. */
  async function openDebugReport() {
    const params = new URLSearchParams(location.search);
    const name = params.get('debugReport');
    const state = params.get('state');
    if ((!name && !state) || !import.meta.env.DEV) return;
    try {
      app.start({ kind: 'load', saveId: name ? await loadDebugReport(name) : await fetchState(state!) });
    } catch (err) {
      console.error(err);
      toast(err instanceof Error ? err.message : String(err), 6000);
    }
  }

  const titles = { settings: 'Settings', load: 'Load game', credits: 'Credits' } as const;

  $effect(() => {
    if (app.screen === 'game') void loadGameView();
  });

  /**
   * The live 3D town behind the menus: a showcase neighbourhood on the main menu (and its
   * overlays), the household's new town behind character creation. The neighbourhood and
   * home screens show and frame their town themselves.
   */
  $effect(() => {
    if (!app.dataReady || app.fatal || app.booting) return;
    const screen = app.screen;
    const town = app.town;
    let cancelled = false;
    if (screen === 'menu' || screen === 'create') {
      void (async () => {
        const draft = screen === 'create' && town ? town : await menuScene.showcaseDraft();
        if (cancelled) return;
        menuScene.highlight({});
        menuScene.shoot({ kind: screen }, screen === 'menu' ? 2.4 : 2);
        await menuScene.show(draft).catch((err: unknown) => console.warn('[menu] town preview failed', err));
        // "New game" starts from this neighbourhood: have its residents' portraits ready by then.
        if (!cancelled && screen === 'menu') {
          services.previews?.prefetch(draft.households.flatMap((h) => h.household.members.map((m) => ({ gender: m.gender, appearance: m.appearance }))));
        }
      })();
    }
    return () => {
      cancelled = true;
    };
  });

  // Live render settings (style, effects) while no game is running.
  $effect(() => {
    const s = $state.snapshot(settings);
    if (app.screen !== 'game') host.applyMenuSettings(s);
  });

  // Full-screen menus cover the view: nothing is drawn behind them (their frosted glass keeps the last frame).
  $effect(() => {
    host.cover(app.overlay !== null || app.crash !== null || (app.screen === 'game' && (game.pauseMenu || (game.plannerOpen && game.mode === 'live'))));
  });

  /**
   * Prepare the next game in the background so starting it is instant: the latest save
   * while on the main menu, the chosen house while picking a home, the chosen household while
   * picking one to play. All build hidden behind the menus' town.
   */
  $effect(() => {
    if (!app.dataReady || app.booting) return;
    const screen = app.screen;
    const slot = app.homeSlot;
    const home = slot === null ? null : app.homeRequest(slot);
    const play = app.playSlot === null ? null : app.playRequest(app.playSlot);
    const snapshot = $state.snapshot(settings);
    let cancelled = false;
    // On the main menu, let the entrance animation and the camera's glide finish first (the
    // latest save is not urgent). Picking a home or a household is: start at once, the build
    // yields between its slices so the camera keeps moving.
    const settle = screen === 'menu';
    const timer = setTimeout(async () => {
      if (settle) await menuScene.settled();
      if (cancelled) return;
      // (Not when the page starts straight into a game or the benchmark.)
      if (screen === 'menu' && !['debugReport', 'state', 'bench'].some((k) => new URLSearchParams(location.search).has(k))) {
        // A save that can't be opened is set aside (and the player told), and the next one tried.
        for (;;) {
          const latest = await latestSave().catch(() => undefined);
          if (cancelled || !latest || app.screen !== 'menu') return;
          try {
            await host.prepare({ kind: 'load', saveId: latest.id }, snapshot, false);
            return;
          } catch (err) {
            if (!(err instanceof UnloadableSave)) return;
            // Unless Continue was clicked meanwhile: then the game screen says it.
            const shown = app.request?.kind === 'load' && app.request.saveId === err.saveId;
            if (!shown) toast(err.message, 7000);
          }
        }
      } else if (screen === 'home' && home) {
        await host.prepare(home, snapshot).catch(() => {});
      } else if (screen === 'play' && play) {
        await host.prepare(play, snapshot).catch(() => {});
      }
    }, settle ? 500 : 250);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });
</script>

<!-- The game canvas lives here permanently (engine and shaders survive between games). -->
<div class="game-layer" class:hidden={app.screen !== 'game' && !app.liveBackdrop} bind:this={gameLayer}></div>

{#if app.fatal}
  <MenuBackdrop />
  <CrashScreen crash={{ kind: 'data', detail: app.fatal }} />
{:else if app.booting}
  <LoadingScreen progress={app.boot} />
{:else if app.screen === 'game'}
  {#if GameView}
    {#key app.sessionKey}
      <svelte:boundary onerror={interfaceFailed}>
        <GameView />
        {#snippet failed()}{/snippet}
      </svelte:boundary>
    {/key}
  {:else}
    <LoadingScreen progress={app.loading ?? { label: 'Moving in', value: 0 }} />
  {/if}
{:else}
  {#if !app.liveBackdrop}<MenuBackdrop />{/if}
  {#if app.screen === 'world'}
    <CreateWorld />
  {:else if app.screen === 'neighbourhood'}
    <CreateNeighbourhood />
  {:else if app.screen === 'create'}
    <CreateHousehold />
  {:else if app.screen === 'home'}
    <ChooseHome />
  {:else if app.screen === 'play'}
    <PlayHousehold />
  {:else}
    <MainMenu />
  {/if}
{/if}

{#if app.overlay}
  <Modal title={titles[app.overlay]} width={app.overlay === 'credits' ? 520 : app.overlay === 'settings' ? 880 : 680} scroll={app.overlay !== 'settings'} onclose={() => (app.overlay = null)}>
    {#if app.overlay === 'settings'}<SettingsPanel />{:else if app.overlay === 'load'}<LoadGame />{:else}<Credits />{/if}
  </Modal>
{/if}

{#if app.crash && !app.fatal}<CrashScreen crash={app.crash} />{/if}

{#if app.newVersion}
  <div class="scaled update">
    <div class="glass bar" role="alert">
      <span>A new version is available.</span>
      <button class="btn primary" onclick={() => location.reload()}>Reload</button>
    </div>
  </div>
{/if}

<div class="scaled toast-layer"><Toasts /></div>

<style>
  .game-layer {
    position: fixed;
    inset: 0;
    transition: opacity 600ms var(--ease);
  }
  .game-layer.hidden {
    opacity: 0;
    pointer-events: none;
  }
  .game-layer :global(.scene) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
    outline: none;
    touch-action: none;
  }
  .game-layer :global(.world-overlay) {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  /* Fades the scene out and in when it changes completely (menu town <-> a saved game). */
  .game-layer :global(.scene-veil) {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(ellipse at 60% 40%, #f3e6d2 0%, #d9cbb8 60%, #b9ab98 100%);
    opacity: 0;
    transition: opacity 240ms var(--ease);
  }
  .game-layer :global(.scene-veil.on) {
    opacity: 1;
  }
  /* "A new version is available": a bar at the top, over everything but the toasts. */
  .update {
    position: fixed;
    inset: 16px 0 auto 0;
    display: grid;
    place-items: center;
    pointer-events: none;
    z-index: 55;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 8px 8px 20px;
    border-radius: var(--radius-pill);
    font-weight: 600;
    pointer-events: auto;
  }
  .bar .btn {
    border-radius: var(--radius-pill);
  }
  .toast-layer {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 60;
  }
</style>
