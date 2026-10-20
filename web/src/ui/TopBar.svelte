<script lang="ts">
  import Icon from './Icon.svelte';
  import { clock, money, WEEKDAYS } from './format';
  import { settings } from '../settings/settings.svelte';
  import { services } from './services';
  import { game, nextWallMode } from './state.svelte';

  const speeds = [
    { value: 0, icon: 'icon.ui.pause', label: 'Pause (Space)' },
    { value: 1, icon: 'icon.ui.speed1', label: 'Normal speed (1)' },
    { value: 2, icon: 'icon.ui.speed2', label: 'Fast (2)' },
    { value: 3, icon: 'icon.ui.speed3', label: 'Faster (3)' },
    { value: 4, icon: 'icon.ui.speed4', label: 'Time-lapse (4)' },
    { value: 5, icon: 'icon.ui.speed5', label: 'Fastest (5)' },
  ];
  const modes = [
    { id: 'live', icon: 'icon.ui.live', label: 'Live', title: 'Live (L): the household and the world, in motion' },
    { id: 'build', icon: 'icon.ui.build', label: 'Build', title: 'Build (B): furnish, build, and see how the home is doing' },
  ] as const;
  const rent = services.content.economy.rent;
  function toggleTown() {
    game.townOpen = !game.townOpen;
    if (game.townOpen) game.journalOpen = false;
  }
  const fundsTitle = $derived(
    game.creative
      ? 'Household funds · Creative game: building and buying are free, and your home pays no rent or bills.'
      : game.rent !== null && rent
      ? `Household funds · rent ${money(game.rent)} + bills ${money(game.bills ?? 0)} every ${WEEKDAYS[rent.weekday]} at ${clock(rent.hour * 60, settings.clock24h)}. Bills grow with the value of everything you own and with how many of you there are.`
      : 'Household funds',
  );
</script>

<header class="bar">
  <div class="glass group">
    <span class="brand">{game.household || 'Idyll Lives'}</span>
    <div class="segmented" role="tablist" aria-label="Mode">
      {#each modes as mode (mode.id)}
        <button role="tab" aria-selected={mode.id === game.mode} class:active={mode.id === game.mode} title={mode.title} onclick={() => services.controls.setMode(mode.id)}>
          <Icon name={mode.icon} size={16} />
          <span>{mode.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="glass group center">
    <div class="time">
      <span class="day tabular">{WEEKDAYS[game.weekday]} · Day {game.day}</span>
      <span class="clock tabular">{clock(game.minute, settings.clock24h)}</span>
    </div>
    {#if game.mode !== 'live'}
      <span class="paused" title="Time stands still while you build. Back to Live (L) to resume.">
        <Icon name="icon.ui.pause" size={14} />Paused
      </span>
    {/if}
    {#if game.mode === 'build'}
      <button
        class="undo"
        disabled={game.undoSteps === 0}
        title={game.undoSteps ? 'Undo the last change (Ctrl+Z / ⌘Z)' : 'Nothing to undo. Changes can be undone until time moves on.'}
        onclick={() => services.controls.undo()}
      >
        <Icon name="icon.ui.undo" size={16} /><span>Undo</span>
      </button>
      <button
        class="undo redo"
        disabled={game.redoSteps === 0}
        title={game.redoSteps ? 'Redo what you undid (Ctrl+Shift+Z / ⌘⇧Z)' : 'Nothing to redo'}
        aria-label="Redo"
        onclick={() => services.controls.redo()}
      >
        <span class="mirror"><Icon name="icon.ui.undo" size={16} /></span>
      </button>
      <button
        class="undo redo"
        class:on={game.eyedropper}
        aria-pressed={game.eyedropper}
        title="Eyedropper (E, or hold Alt): click a wall, floor, door, window or object to build or buy more of the same"
        aria-label="Eyedropper"
        onclick={() => services.controls.toggleEyedropper()}
      >
        <Icon name="icon.ui.eyedropper" size={16} />
      </button>
    {:else if game.mode === 'live'}
      <div class="segmented">
        {#each speeds as s (s.value)}
          <button class:active={game.speed === s.value} title={s.label} aria-label={s.label} onclick={() => services.controls.setSpeed(s.value)}>
            <Icon name={s.icon} size={16} />
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="glass group">
    {#if game.mode === 'live'}
      {#if game.occupied}
        <button class="tool" class:active={game.plannerOpen} title="Planner (P): routines and goals" aria-label="Planner" onclick={() => services.controls.openPlanner()}>
          <Icon name="icon.ui.calendar" />
        </button>
      {/if}
      <button class="tool" class:active={game.journalOpen} title="Journal (J)" aria-label="Journal" aria-pressed={game.journalOpen} onclick={() => services.controls.toggleJournal()}>
        <Icon name="icon.ui.journal" />
      </button>
      <button class="tool" class:active={game.townOpen} title="Town map (M)" aria-label="Town map" aria-pressed={game.townOpen} onclick={toggleTown}>
        <Icon name="icon.ui.town" />
      </button>
      <span class="divider" aria-hidden="true"></span>
    {/if}
    <span class="funds tabular" class:debt={game.funds < 0} title={fundsTitle}>
      <Icon name="icon.ui.funds" size={16} />{money(game.funds)}
    </span>
    {#if game.storeys - game.lowestStorey > 1}
      <div class="storeys" role="group" aria-label="Storey">
        <button class="tool" title="Storey down (Page Down)" aria-label="Storey down" disabled={game.storey <= game.lowestStorey} onclick={() => services.controls.setStorey(game.storey - 1)}>
          <span class="arrow down"><Icon name="icon.ui.back" size={16} /></span>
        </button>
        <span class="storey" title="Storey in view: higher ones are hidden">{game.storey < 0 ? 'Basement' : game.storey === 0 ? 'Ground' : `Floor ${game.storey}`}</span>
        <button class="tool" title="Storey up (Page Up)" aria-label="Storey up" disabled={game.storey >= game.storeys - 1} onclick={() => services.controls.setStorey(game.storey + 1)}>
          <span class="arrow up"><Icon name="icon.ui.back" size={16} /></span>
        </button>
      </div>
    {/if}
    <button
      class="tool"
      title="Walls: {game.wallMode} (W: up → cutaway → down)"
      aria-label="Toggle walls"
      onclick={() => services.controls.setWallMode(nextWallMode(game.wallMode))}
    >
      <Icon name={game.wallMode === 'up' ? 'icon.ui.wallsUp' : 'icon.ui.wallsDown'} />
    </button>
    <button class="tool" title="Menu (Esc)" aria-label="Pause menu" onclick={() => services.controls.openPauseMenu()}>
      <Icon name="icon.ui.menu" />
    </button>
  </div>
</header>

<style>
  /* Three columns, the middle one centred on the screen: the clock and speeds stay put when the
     groups beside them change width (the funds as money comes and goes). */
  .bar {
    position: absolute;
    inset: var(--edge) var(--edge) auto;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: start;
    gap: var(--gap);
    pointer-events: none;
  }
  .bar > .group:first-child {
    justify-self: start;
  }
  .bar > .group:last-child {
    justify-self: end;
  }
  .group {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px;
    pointer-events: auto;
  }
  .brand {
    font-weight: 700;
    letter-spacing: -0.02em;
    padding: 0 8px 0 10px;
  }
  .segmented {
    display: flex;
    gap: 2px;
    padding: 3px;
    background: var(--hairline);
    border-radius: var(--radius-md);
  }
  .segmented button {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-weight: 550;
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .segmented button:hover:not(:disabled) {
    color: var(--text);
  }
  .segmented button.active {
    background: var(--glass-strong);
    color: var(--accent);
    box-shadow: var(--shadow-sm);
  }
  .segmented button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .time {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
    padding: 0 6px 0 10px;
    /* As wide as "Mon · Day 99" or "12:00 AM", so the speeds beside it don't move as the day or the hour changes. */
    min-width: 104px;
    box-sizing: border-box;
  }
  .day {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .clock {
    font-size: 18px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .funds {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px 0 8px;
    font-weight: 650;
    color: var(--good);
  }
  .funds.debt {
    color: var(--bad);
  }
  .tool {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .divider {
    width: 1px;
    height: 22px;
    background: var(--hairline);
  }
  .tool:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .storeys {
    display: flex;
    align-items: center;
  }
  .storeys .tool {
    width: 28px;
  }
  .storey {
    min-width: 50px;
    text-align: center;
    font-size: 12px;
    font-weight: 650;
    color: var(--text-muted);
  }
  .arrow {
    display: inline-flex;
  }
  .arrow.up {
    transform: rotate(90deg);
  }
  .arrow.down {
    transform: rotate(-90deg);
  }
  .undo.redo {
    padding: 0 10px;
  }
  .undo.on {
    background: var(--accent);
    color: #fff;
  }
  .mirror {
    display: inline-flex;
    transform: scaleX(-1);
  }
  .undo {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 36px;
    padding: 0 12px;
    border-radius: var(--radius-pill);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    color: var(--text);
    font-size: 12px;
    font-weight: 650;
    white-space: nowrap;
    transition:
      transform var(--fast) var(--ease),
      opacity var(--fast) var(--ease);
  }
  .undo:hover:not(:disabled) {
    transform: translateY(-1px);
  }
  .undo:active:not(:disabled) {
    transform: scale(0.96);
  }
  .undo:disabled {
    opacity: 0.45;
    cursor: default;
    box-shadow: none;
  }
  .paused {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 36px;
    padding: 0 12px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 12px;
    font-weight: 650;
    white-space: nowrap;
  }
  .tool:hover:not(:disabled),
  .tool.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
