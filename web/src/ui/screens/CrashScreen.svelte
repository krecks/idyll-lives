<script lang="ts">
  import { host } from '../../game/host';
  import { app, type Crash } from '../app.svelte';
  import { saveDebugReport } from '../debugReport';
  import MenuItem from '../kit/MenuItem.svelte';
  import { menuArrows } from '../kit/menuKeys';
  import { services } from '../services';
  import { game } from '../state.svelte';

  /**
   * The game can't go on: says so plainly and offers a way out, without needing the simulation
   * (it may be gone): reload, save a debug report, or go back to the menu. When only drawing or
   * the interface broke, the game still runs and can be saved before reloading.
   */
  let { crash }: { crash: Crash } = $props();

  const kind = $derived(crash.kind);
  /** The game being played still runs (the simulation didn't fail): it can be saved first. */
  const canSave = host.canSave;
  /** Without game data there are no icons. */
  const icons = !!services.assets;
  let saving = $state(false);
  let saveError = $state('');

  const title = $derived(
    kind === 'graphics'
      ? 'Graphics were reset'
      : kind === 'data'
        ? "Couldn't load game data"
        : kind === 'interface'
          ? "Something went wrong in the game's interface"
          : kind === 'render' && canSave
            ? 'The picture stopped because of an error'
            : 'The game stopped because of an error',
  );
  const text = $derived(
    kind === 'data'
      ? 'Check your connection, then reload.'
      : kind === 'graphics'
        ? canSave
          ? "Your graphics card was reset, so the picture can't go on. Your game still runs: save it and reload to carry on."
          : "Your graphics card was reset, so the picture can't go on. Reload to carry on."
        : canSave && kind !== 'sim'
          ? 'Your game still runs: save it and reload to carry on.'
          : 'Your last save is safe.',
  );

  async function saveAndReload() {
    saving = true;
    saveError = '';
    try {
      await host.saveAndReload();
    } catch (err) {
      saveError = `Saving failed: ${err instanceof Error ? err.message : String(err)}`;
      saving = false;
    }
  }

  /** The running game's report (with its save while it runs), or one without a game. */
  function report() {
    void saveDebugReport('', (note) => (app.screen === 'game' && services.controls ? services.controls.debugReport(note) : host.basicReport(note)));
  }

  function toMenu() {
    app.crash = null;
    app.toMenu();
  }
</script>

<div class="crash scaled">
  <div class="scrim"></div>
  <div class="panel" role="alertdialog" aria-modal="true" aria-labelledby="crash-title" aria-describedby="crash-text">
    <h2 id="crash-title">{title}</h2>
    <p id="crash-text">{text}</p>
    {#if kind === 'data'}
      <p class="detail">{crash.detail}</p>
    {:else if crash.detail}
      <details>
        <summary>What happened</summary>
        <p class="detail">{crash.detail}</p>
      </details>
    {/if}
    {#if saveError}<p class="error">{saveError}</p>{/if}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <nav onkeydown={menuArrows}>
      {#if canSave && kind !== 'sim' && kind !== 'data'}
        <MenuItem label={saving ? 'Saving…' : 'Save and reload'} icon={icons ? 'icon.ui.save' : undefined} variant="primary" disabled={saving} onclick={saveAndReload} />
        <MenuItem label="Reload without saving" icon={icons ? 'icon.ui.rotate' : undefined} disabled={saving} onclick={() => location.reload()} />
      {:else}
        <MenuItem label="Reload" icon={icons ? 'icon.ui.rotate' : undefined} variant="primary" onclick={() => location.reload()} />
      {/if}
      <MenuItem label={game.debugSaving ? 'Saving the report…' : 'Save a debug report'} icon={icons ? 'icon.ui.info' : undefined} disabled={game.debugSaving} onclick={report} />
      {#if kind === 'sim' || kind === 'interface'}
        <MenuItem label="Main menu" variant="quiet" disabled={saving} onclick={toMenu} />
      {/if}
    </nav>
  </div>
</div>

<style>
  .crash {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 50;
  }
  .scrim {
    position: absolute;
    inset: 0;
    background: var(--backdrop);
    backdrop-filter: blur(8px) saturate(0.8);
    -webkit-backdrop-filter: blur(8px) saturate(0.8);
    animation: fade var(--slow) var(--ease);
  }
  /* The pause menu's dark glass. */
  .panel {
    position: relative;
    width: 420px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 28px 24px 22px;
    border-radius: 28px;
    color: #f6f2ea;
    background: linear-gradient(180deg, rgba(22, 27, 38, 0.78), rgba(14, 18, 26, 0.72));
    border: 1px solid var(--glass-dark-border);
    box-shadow:
      0 40px 100px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(26px) saturate(1.3);
    -webkit-backdrop-filter: blur(26px) saturate(1.3);
    animation: rise var(--slow) var(--ease);
  }
  h2 {
    margin: 0;
    padding: 0 6px;
    font-size: 22px;
    letter-spacing: -0.02em;
  }
  p {
    margin: 0 0 8px;
    padding: 0 6px;
    color: rgba(246, 242, 234, 0.78);
  }
  details {
    margin: 0 0 10px;
    padding: 0 6px;
    color: rgba(246, 242, 234, 0.6);
    font-size: 13px;
  }
  summary {
    cursor: pointer;
  }
  .detail {
    margin: 6px 0 0;
    padding: 0;
    max-height: 120px;
    overflow: auto;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
    color: rgba(246, 242, 234, 0.6);
    user-select: text;
  }
  .panel > .detail {
    margin-bottom: 8px;
    padding: 0 6px;
  }
  .error {
    color: #ff9d8f;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 6px;
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.98);
    }
  }
</style>
