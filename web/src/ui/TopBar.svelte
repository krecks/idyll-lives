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
  const fundsTitle = $derived.by(() => {
    if (game.creative) return 'Household funds · Creative game: building and buying are free, and your home pays no rent or bills.';
    if (game.rent !== null && rent) {
      return `Household funds · rent ${money(game.rent)} + bills ${money(game.bills ?? 0)} every ${WEEKDAYS[rent.weekday]} at ${clock(rent.hour * 60, settings.clock24h)}. Bills grow with the value of everything you own and with how many of you there are.`;
    }
    return 'Household funds';
  });

  function storeyLabel(storey: number): string {
    if (storey < 0) return 'Basement';
    if (storey === 0) return 'Ground';
    return `Floor ${storey}`;
  }
</script>

<header class="bar" class:live={game.mode === 'live'}>
  <div class="zone glass">
    <span class="brand" title={game.household || undefined}>{game.household || 'Idyll Lives'}</span>
    <div class="segmented modes" role="tablist" aria-label="Mode">
      {#each modes as mode (mode.id)}
        <button role="tab" aria-selected={mode.id === game.mode} class:active={mode.id === game.mode} title={mode.title} onclick={() => services.controls.setMode(mode.id)}>
          <Icon name={mode.icon} size={16} />
          <span class="label">{mode.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="zone glass">
    <div class="time">
      <span class="day tabular">{WEEKDAYS[game.weekday]} · Day {game.day}</span>
      <span class="clock tabular">{clock(game.minute, settings.clock24h)}</span>
    </div>
    {#if game.mode === 'live'}
      <div class="segmented speeds">
        {#each speeds as s (s.value)}
          <button class:active={game.speed === s.value} title={s.label} aria-label={s.label} onclick={() => services.controls.setSpeed(s.value)}>
            <Icon name={s.icon} size={15} />
          </button>
        {/each}
      </div>
    {:else}
      <span class="paused" title="Time stands still while you build. Back to Live (L) to resume.">
        <Icon name="icon.ui.pause" size={13} />Paused
      </span>
    {/if}
  </div>

  <div class="zone glass right">
    {#if game.mode === 'live'}
      <div class="tools">
        {#if game.occupied}
          <button class="tool" class:active={game.plannerOpen} title="Planner (P): routines and goals" aria-label="Planner" onclick={() => services.controls.openPlanner()}>
            <Icon name="icon.ui.calendar" />
          </button>
        {/if}
        <button class="tool" class:active={game.journalOpen} title="Journal (J)" aria-label="Journal" aria-pressed={game.journalOpen} onclick={() => services.controls.toggleJournal()}>
          <Icon name="icon.ui.journal" />
        </button>
        <button class="tool" class:active={game.townOpen} title="Town map (M)" aria-label="Town map" aria-pressed={game.townOpen} aria-controls="town-map" onclick={toggleTown}>
          <Icon name="icon.ui.town" />
        </button>
      </div>
      <span class="divider" aria-hidden="true"></span>
    {/if}

    <span class="funds tabular" class:debt={game.funds < 0} title={fundsTitle}>
      <Icon name="icon.ui.funds" size={16} />{money(game.funds)}
    </span>

    {#if game.storeys - game.lowestStorey > 1}
      <div class="storeys" role="group" aria-label="Storey">
        <button class="tool small" title="Storey down (Page Down)" aria-label="Storey down" disabled={game.storey <= game.lowestStorey} onclick={() => services.controls.setStorey(game.storey - 1)}>
          <span class="arrow down"><Icon name="icon.ui.back" size={14} /></span>
        </button>
        <span class="storey" title="Storey in view: higher ones are hidden">{storeyLabel(game.storey)}</span>
        <button class="tool small" title="Storey up (Page Up)" aria-label="Storey up" disabled={game.storey >= game.storeys - 1} onclick={() => services.controls.setStorey(game.storey + 1)}>
          <span class="arrow up"><Icon name="icon.ui.back" size={14} /></span>
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
  /* Three islands of glass with the game between them: who you are and the mode on the left, the
     clock in the middle (centred on the screen, so it doesn't move as the funds or the buttons
     beside it change), and what you can do now on the right. When the sides are too wide to
     centre it, the clock moves aside rather than slip under them. */
  .bar {
    position: absolute;
    inset: var(--edge) var(--edge) auto;
    display: grid;
    grid-template-columns: minmax(max-content, 1fr) auto minmax(max-content, 1fr);
    align-items: center;
    gap: var(--gap);
    pointer-events: none;
    container-type: inline-size;
  }
  .zone {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 44px;
    padding: 0 4px;
    border-radius: var(--radius-md);
    pointer-events: auto;
  }
  .zone:first-child {
    justify-self: start;
  }
  .zone:nth-child(2) {
    justify-self: center;
  }
  .right {
    justify-self: end;
  }
  .brand {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 6px 0 8px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .segmented {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--hairline);
  }
  .segmented button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-weight: 600;
    font-size: 13px;
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .segmented button:hover {
    color: var(--text);
  }
  .modes button.active {
    background: var(--text);
    color: var(--text-inverse);
  }
  .speeds {
    margin-left: 4px;
  }
  .speeds button {
    width: 28px;
    padding: 0;
    justify-content: center;
  }
  .speeds button.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow-sm);
  }
  .time {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
    padding: 0 6px 0 8px;
    min-width: 100px;
  }
  .day {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .clock {
    font-size: 17px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .paused {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 11px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 12px;
    font-weight: 650;
    white-space: nowrap;
  }
  .tools {
    display: flex;
    gap: 2px;
  }
  .tool {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  .tool.small {
    width: 26px;
  }
  .tool:hover:not(:disabled),
  .tool.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .tool:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .divider {
    width: 1px;
    height: 20px;
    margin: 0 2px;
    background: var(--hairline);
  }
  .funds {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    font-weight: 650;
    white-space: nowrap;
    color: var(--good);
  }
  .funds.debt {
    color: var(--bad);
  }
  .storeys {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .storey {
    min-width: 48px;
    text-align: center;
    font-size: 11.5px;
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
  /* Narrow windows: the household's name goes first, then the mode labels; the clock keeps its place. */
  @container (max-width: 1180px) {
    .brand {
      display: none;
    }
  }
  /* In live mode the right-hand tools are widest, so the storey's name goes first there. */
  @container (max-width: 1100px) {
    .live .storey {
      display: none;
    }
  }
  @container (max-width: 960px) {
    .modes .label {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
      white-space: nowrap;
    }
  }
  @container (max-width: 940px) {
    .zone {
      gap: 2px;
    }
    .tool {
      width: 30px;
    }
    .funds {
      padding: 0 5px;
    }
    .modes button {
      padding: 0 8px;
    }
  }
  /* Too narrow for one row even so (a small window or a large interface scale): the clock and
     speeds take a row of their own. The width: the bar at its widest, in Live with the planner,
     the storeys and a five-figure balance. (Of the HUD: the bar can't query its own width.
     Hud.svelte moves what hangs below the bar at the same width.) */
  @container hud (max-width: 780px) {
    .bar {
      grid-template-columns: auto 1fr;
      row-gap: 8px;
    }
    .zone:nth-child(2) {
      grid-column: 1 / -1;
      grid-row: 2;
      justify-self: center;
    }
  }
</style>
