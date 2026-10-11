<script lang="ts">
  import type { GameSession } from '../../game/session';
  import { settings } from '../../settings/settings.svelte';
  import { app } from '../app.svelte';
  import { clock } from '../format';
  import MenuItem from '../kit/MenuItem.svelte';
  import { menuArrows } from '../kit/menuKeys';
  import { game, toast } from '../state.svelte';

  let { session }: { session: GameSession } = $props();

  let view = $state<'main' | 'save' | 'quit'>('main');
  let saveName = $state('');
  let busy = $state(false);

  function openSave() {
    saveName = `${session.household} · Day ${game.day}`;
    view = 'save';
  }

  async function save(): Promise<boolean> {
    busy = true;
    try {
      await session.save(saveName.trim() || session.household);
      toast('Game saved');
      return true;
    } catch (err) {
      toast(`Saving failed: ${err instanceof Error ? err.message : err}`);
      return false;
    } finally {
      busy = false;
    }
  }

  async function saveAndQuit() {
    saveName ||= `${session.household} · Day ${game.day}`;
    if (await save()) app.toMenu();
  }
</script>

<div class="pause scaled">
  <div class="scrim"></div>
  <div class="panel" role="dialog" aria-modal="true" aria-label="Pause menu">
    {#if view === 'main'}
      <span class="eyebrow">Paused</span>
      <h2>{session.household}</h2>
      <p class="when">Day {game.day} · {clock(game.minute, settings.clock24h)}</p>
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <nav onkeydown={menuArrows}>
        <MenuItem label="Resume" icon="icon.ui.speed1" variant="primary" onclick={() => (game.pauseMenu = false)} />
        <MenuItem label="Save game" icon="icon.ui.save" onclick={openSave} />
        <MenuItem label="Load game" icon="icon.ui.load" onclick={() => (app.overlay = 'load')} />
        <MenuItem label="Settings" icon="icon.ui.settings" onclick={() => (app.overlay = 'settings')} />
        <MenuItem label="Quit to main menu" variant="quiet" onclick={() => (view = 'quit')} />
      </nav>
    {:else if view === 'save'}
      <span class="eyebrow">Save game</span>
      <h2>Name this save</h2>
      <form
        onsubmit={async (e) => {
          e.preventDefault();
          if (await save()) view = 'main';
        }}
      >
        <!-- svelte-ignore a11y_autofocus -->
        <input class="name" bind:value={saveName} maxlength="40" autofocus onkeydown={(e) => e.key === 'Escape' && (view = 'main')} />
        <MenuItem type="submit" label={busy ? 'Saving…' : 'Save'} icon="icon.ui.save" variant="primary" disabled={busy} />
        <MenuItem label="Back" variant="quiet" onclick={() => (view = 'main')} />
      </form>
    {:else}
      <span class="eyebrow">Quit to main menu</span>
      <h2>Leave this game?</h2>
      <p class="when">Progress since your last save will be lost.</p>
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <nav onkeydown={menuArrows}>
        <MenuItem label="Save and quit" icon="icon.ui.save" variant="primary" disabled={busy} onclick={saveAndQuit} />
        <MenuItem label="Quit without saving" icon="icon.ui.close" variant="danger" onclick={() => app.toMenu()} />
        <MenuItem label="Cancel" variant="quiet" onclick={() => (view = 'main')} />
      </nav>
    {/if}
  </div>
</div>

<style>
  .pause {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 40;
  }
  .scrim {
    position: absolute;
    inset: 0;
    background: var(--backdrop);
    backdrop-filter: blur(8px) saturate(0.8);
    -webkit-backdrop-filter: blur(8px) saturate(0.8);
    animation: fade var(--slow) var(--ease);
  }
  .panel {
    position: relative;
    width: 380px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 28px 24px 22px;
    border-radius: 28px;
    color: #f6f2ea;
    /* The title screen's dark glass: the same menu in and out of a game. */
    background: linear-gradient(180deg, rgba(22, 27, 38, 0.72), rgba(14, 18, 26, 0.66));
    border: 1px solid var(--glass-dark-border);
    box-shadow:
      0 40px 100px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(26px) saturate(1.3);
    -webkit-backdrop-filter: blur(26px) saturate(1.3);
    animation: rise var(--slow) var(--ease);
  }
  .eyebrow {
    padding: 0 6px;
    color: rgba(246, 242, 234, 0.6);
  }
  h2 {
    margin: 0;
    padding: 0 6px;
    font-size: 24px;
    letter-spacing: -0.02em;
  }
  .when {
    margin: 0 0 14px;
    padding: 0 6px;
    color: rgba(246, 242, 234, 0.72);
  }
  nav,
  form {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  form {
    margin-top: 12px;
  }
  .name {
    height: 48px;
    margin-bottom: 10px;
    padding: 0 16px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.18);
    background: rgba(255, 255, 255, 0.08);
    color: inherit;
    font: inherit;
    font-size: 16px;
    outline: none;
    user-select: text;
  }
  .name:focus {
    border-color: rgba(255, 255, 255, 0.5);
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.12);
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
