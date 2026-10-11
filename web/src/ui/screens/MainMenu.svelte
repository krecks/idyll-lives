<script lang="ts">
  import { onMount } from 'svelte';
  import { menuScene } from '../../game/menuScene';
  import { listSaves, type SaveMeta } from '../../persistence/saves';
  import { settings } from '../../settings/settings.svelte';
  import { app } from '../app.svelte';
  import { clock, timeAgo } from '../format';
  import MenuItem from '../kit/MenuItem.svelte';
  import { enterMenu, menuArrows } from '../kit/menuKeys';

  let saves = $state<SaveMeta[]>([]);
  const latest = $derived(saves[0]);
  /** The graphics API the game draws with: WebGPU where there is one, unless WebGL2 is asked for. */
  const webgpu = $derived(
    typeof navigator !== 'undefined' && 'gpu' in navigator && settings.renderer !== 'webgl' && new URLSearchParams(location.search).get('renderer') !== 'webgl',
  );
  let nav = $state<HTMLElement>();
  /** The neighbourhood drifting behind the menu. */
  let townName = $state('');

  onMount(() => {
    listSaves()
      .then((s) => (saves = s))
      .catch(() => (saves = []));
    menuScene
      .showcaseDraft()
      .then((d) => (townName = d.name))
      .catch(() => {});
  });
</script>

<svelte:window onkeydown={(e) => enterMenu(e, nav)} />

<div class="menu scaled">
  <div class="scrim" aria-hidden="true"></div>

  <section class="panel">
    <header class="brand">
      <h1>Idyll Lives</h1>
      <p>A little life, simulated in your browser.</p>
    </header>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->

    <nav bind:this={nav} onkeydown={menuArrows}>
      {#if latest}
        <button class="continue" onclick={() => app.start({ kind: 'load', saveId: latest.id })}>
          {#if latest.thumbnail}<img class="thumb" src={latest.thumbnail} alt="" />{:else}<span class="thumb blank"></span>{/if}
          <span class="what">
            <span class="label">Continue</span>
            <span class="meta">{latest.household} · Day {latest.day}, {clock(latest.minute, settings.clock24h)}</span>
            <span class="when">Saved {timeAgo(latest.savedAt)}</span>
          </span>
          <span class="arrow" aria-hidden="true">→</span>
        </button>
      {/if}
      <MenuItem label="New game" icon="icon.ui.plus" variant={latest ? 'default' : 'primary'} arrow={!latest} onclick={() => app.newGame()} />
      <MenuItem label="Load game" icon="icon.ui.load" disabled={saves.length === 0} badge={saves.length || undefined} onclick={() => (app.overlay = 'load')} />
      <MenuItem label="Settings" icon="icon.ui.settings" onclick={() => (app.overlay = 'settings')} />
      <MenuItem label="Credits" variant="quiet" onclick={() => (app.overlay = 'credits')} />
    </nav>

    <footer>
      <span>v{__APP_VERSION__}</span>
      <span class="dot"></span>
      <span>{webgpu ? 'WebGPU' : 'WebGL2'}</span>
    </footer>
  </section>

  {#if townName}
    <div class="postcard" aria-hidden="true">
      <span class="sun"></span>
      <span class="where">
        <b>{townName}</b>
        <span>Golden hour</span>
      </span>
    </div>
  {/if}
</div>

<style>
  .menu {
    position: relative;
    height: 100vh;
    display: flex;
    align-items: center;
    color: #f6f2ea;
  }
  /* Darkens the left of the scene behind the panel, and the bottom edge, for contrast. */
  .scrim {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background:
      linear-gradient(90deg, rgba(8, 11, 18, 0.58) 0%, rgba(8, 11, 18, 0.32) 28%, rgba(8, 11, 18, 0) 52%),
      linear-gradient(0deg, rgba(8, 11, 18, 0.42) 0%, rgba(8, 11, 18, 0) 26%),
      linear-gradient(180deg, rgba(8, 11, 18, 0.22) 0%, rgba(8, 11, 18, 0) 18%);
  }
  .panel {
    position: relative;
    width: min(420px, 36vw);
    min-width: 360px;
    margin-left: max(32px, 5vw);
    padding: 40px 32px 26px;
    display: flex;
    flex-direction: column;
    gap: 24px;
    border-radius: 28px;
    background: linear-gradient(180deg, rgba(22, 27, 38, 0.56), rgba(14, 18, 26, 0.5));
    border: 1px solid var(--glass-dark-border);
    box-shadow:
      0 40px 100px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(26px) saturate(1.3);
    -webkit-backdrop-filter: blur(26px) saturate(1.3);
    animation: enter 700ms var(--ease) both;
  }
  .brand {
    padding: 0 6px;
  }
  h1 {
    margin: 0;
    font-size: 44px;
    font-weight: 750;
    letter-spacing: -0.04em;
    line-height: 1;
    text-shadow: 0 2px 18px rgba(0, 0, 0, 0.35);
  }
  p {
    margin: 8px 0 0;
    color: rgba(246, 242, 234, 0.78);
    font-size: 15px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .continue {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 8px;
    padding: 10px 18px 10px 10px;
    border-radius: 18px;
    text-align: left;
    background: linear-gradient(180deg, #ffffff, #eef1f6);
    color: #121826;
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.32);
    transition:
      transform var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
    animation: enter 600ms var(--ease) both;
  }
  .continue:hover,
  .continue:focus-visible {
    transform: translateY(-1px);
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.38);
  }
  .thumb {
    width: 92px;
    height: 58px;
    flex: none;
    border-radius: 12px;
    object-fit: cover;
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
  }
  .thumb.blank {
    background: linear-gradient(160deg, #9cc1e6, #e7c9a3 60%, #6d9150 61%);
  }
  .what {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .continue .label {
    font-size: 19px;
    font-weight: 750;
    letter-spacing: -0.01em;
  }
  .continue .meta {
    font-size: 13px;
    font-weight: 600;
    color: #3b4254;
  }
  .when {
    font-size: 12px;
    color: #5a6274;
  }
  .continue .arrow {
    font-size: 20px;
    transition: transform var(--fast) var(--ease);
  }
  .continue:hover .arrow {
    transform: translateX(3px);
  }
  footer {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 6px;
    color: rgba(246, 242, 234, 0.6);
    font-size: 12px;
  }
  .dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: currentColor;
  }
  /* "Now showing": the town behind the menu, like a postcard caption. */
  .postcard {
    position: absolute;
    right: max(28px, 3vw);
    bottom: 32px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 18px 10px 12px;
    border-radius: var(--radius-pill);
    background: rgba(14, 18, 26, 0.4);
    border: 1px solid var(--glass-dark-border);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    animation: enter 900ms 300ms var(--ease) both;
  }
  .sun {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: radial-gradient(circle at 50% 50%, #fff3cf 0 30%, #ffc56a 31% 55%, rgba(255, 170, 80, 0) 72%);
  }
  .where {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }
  .where b {
    font-size: 14px;
    font-weight: 700;
  }
  .where span {
    font-size: 12px;
    color: rgba(246, 242, 234, 0.72);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(14px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .panel,
    .continue {
      animation: none;
    }
  }
</style>
