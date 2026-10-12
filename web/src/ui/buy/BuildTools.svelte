<script lang="ts">
  import Icon from '../Icon.svelte';
  import { money } from '../format';
  import { services } from '../services';
  import { play } from '../sfx';
  import { game, type BuildTool } from '../state.svelte';
  import Blueprints from './Blueprints.svelte';
  import BuildDock from './BuildDock.svelte';
  import LookPicker from './LookPicker.svelte';

  /**
   * Build mode's Construct tab: walls, rooms, paint, floors, doors, windows, stairs and removing
   * them, on the home lot. A toolbar in groups with the looks for the tool in hand beside it; one
   * line of how-to in the dock's header (what an edit costs shows at the cursor: `BuyFx`).
   */
  const content = services.content;
  const prices = $derived(game.catalog?.build);
  type Tool = { id: BuildTool; label: string; icon: string; unit: string; hint: string; note: string };
  const groups: Tool[][] = [
    [
      { id: 'wall', label: 'Wall', icon: 'icon.ui.wall', unit: '/m', hint: 'Drag along the grid to build · at 45° for a diagonal', note: "Furniture can't stand on tiles a diagonal wall crosses." },
      { id: 'room', label: 'Room', icon: 'icon.ui.room', unit: '/m', hint: 'Drag out a rectangle, then add a door', note: "An empty room can be closed off; residents and furniture can't be shut in." },
      { id: 'fence', label: 'Fence', icon: 'icon.ui.fence', unit: '/m', hint: 'Drag along the grid, then add a gate', note: 'Fences keep a garden outdoors (no floor, no roof); leave a gate so nobody is shut in.' },
      { id: 'gate', label: 'Gate', icon: 'icon.ui.gate', unit: '', hint: 'Click a fence (or open ground) to fit a gate', note: 'Leave a gate so nobody is shut in.' },
      { id: 'stairs', label: 'Stairs', icon: 'icon.ui.stairs', unit: '', hint: 'Click where the stairs start · R turns them · click your stairs to move them', note: 'Build the storey above first. Stairs placed in the basement lead up into a room on the ground floor. Stairs are climbed from the front; nothing can stand on the steps or where they come up.' },
      { id: 'roof', label: 'Roof', icon: 'icon.ui.roof', unit: '', hint: 'Pick a shape and a colour: the whole roof changes at once', note: 'The roof covers every room of the house. Changing it is free.' },
    ],
    [
      { id: 'paint', label: 'Paint', icon: 'icon.skill.creativity', unit: '', hint: 'Click the side of a wall, or drag along walls · Shift-click: a whole room', note: 'Each side of a wall has its own covering.' },
      { id: 'floor', label: 'Floor', icon: 'icon.ui.floor', unit: '', hint: 'Click a tile, or drag over several · Shift-click: a whole room', note: 'Floors go inside rooms; every tile can have its own.' },
    ],
    [
      { id: 'door', label: 'Door', icon: 'icon.ui.door', unit: '', hint: 'Click a wall to fit a door · click one to resize it', note: 'Doors go into full-height walls, straight or diagonal; pick another style to replace one.' },
      { id: 'window', label: 'Window', icon: 'icon.ui.window', unit: '', hint: 'Click a wall to fit a window · click one to resize it', note: 'Windows go into full-height walls, straight or diagonal; pick another style to replace one.' },
    ],
    [
      { id: 'remove', label: 'Remove', icon: 'icon.ui.eraser', unit: '/m', hint: 'Drag along walls to tear them down · click a door or window to wall it up', note: "Residents and furniture can't be shut in." },
      { id: 'move', label: 'Move', icon: 'icon.ui.move', unit: '', hint: 'Drag a room where it should go', note: "The room's walls, doors, windows, floor and furniture come along; walls it shares with another room stay." },
    ],
    [
      { id: 'blueprint', label: 'Blueprints', icon: 'icon.ui.blueprint', unit: '', hint: 'Save the house you built, or build a saved one on a cleared lot', note: 'A blueprint is built all at once, turned to face the street, and paid like building it by hand.' },
    ],
  ];
  const tools = groups.flat();
  /** Tools with nothing to do in the basement (no windows, gardens or roof underground). */
  const aboveGround = new Set<BuildTool>(['window', 'fence', 'gate', 'roof']);
  const underground = (id: BuildTool) => game.storey < 0 && aboveGround.has(id);
  /** What the tool costs with the look picked (per metre for walls, per face for paint, per tile for floors). */
  const price = (id: BuildTool): string => {
    if (!prices) return '';
    if (id === 'blueprint') return '';
    if (game.creative || id === 'roof' || id === 'move') return 'Free';
    if (id === 'stairs') {
      const stairs = content.shop.find((d) => d.stairs)?.price ?? 0;
      return stairs ? money(stairs) : 'Free';
    }
    const look = game.buildLook;
    const p =
      id === 'wall' || id === 'room'
        ? prices.wall
        : id === 'paint'
          ? look.cover
            ? (content.wallCoverings[look.cover - 1]?.price ?? 0)
            : 0
          : id === 'floor'
            ? look.floor
              ? (content.floorCoverings[look.floor - 1]?.price ?? 0)
              : 0
            : id === 'fence'
              ? (content.fenceStyles[look.fence]?.price ?? prices.fence ?? 0)
              : id === 'gate'
                ? (content.fenceStyles[look.fence]?.price ?? prices.fence ?? 0) + (prices.gate ?? 0)
            : id === 'door'
              ? (content.doorStyles[look.door]?.price ?? prices.door)
              : id === 'window'
                ? (content.windowStyles[look.window]?.price ?? prices.window)
                : prices.remove;
    return p ? money(p) : 'Free';
  };
  const costLabel = (t: Tool) => {
    const p = price(t.id);
    return p ? ` · ${p}${p === 'Free' ? '' : t.unit}` : '';
  };
  const diagonal = $derived(
    prices && game.buildTool === 'wall' ? ` A diagonal wall ${game.creative ? 'is free' : `costs ${money(prices.diagonalWall ?? Math.round(prices.wall * 1.414))} a tile`}.` : '',
  );
  const tool = $derived(tools.find((t) => t.id === game.buildTool) ?? tools[0]);
  /** Tools that draw (and so can be cancelled half-way). */
  const draws = $derived(!['roof', 'blueprint', 'stairs'].includes(tool.id));

  function pick(id: BuildTool) {
    play('tab');
    services.controls.setBuildTool(id);
  }
</script>

<BuildDock label="Construct" tipId="construct" tall={game.buildTool === 'blueprint'} style="--build: #c9772b; --build-soft: rgba(201, 119, 43, 0.14)">
  {#snippet tip()}
    <b>Build your home.</b> Pick a tool, then drag on your lot to draw walls and rooms, or click a wall to fit doors and windows. Point at a tool for what it costs; the cursor shows what each edit
    will. Residents and furniture can't be shut in. <kbd>Esc</kbd> or right-click cancels.
  {/snippet}

  {#snippet head()}
    {#key tool.id}
      <p class="hint"><b>{tool.label}</b> {tool.hint}{#if draws}<span class="muted">{' · '}<kbd>Esc</kbd> or right-click cancels</span>{/if}</p>
    {/key}
  {/snippet}

  {#snippet body()}
    <div class="build">
      <div class="tools" role="radiogroup" aria-label="Build tool">
        {#each groups as g, gi (gi)}
          <div class="group">
            {#each g as t (t.id)}
              <button
                role="radio"
                aria-checked={game.buildTool === t.id}
                class:active={game.buildTool === t.id}
                disabled={underground(t.id)}
                title="{t.label}{costLabel(t)} — {underground(t.id) ? 'Not in the basement.' : t.note}{t.id === 'wall' ? diagonal : ''}"
                onclick={() => pick(t.id)}
              >
                <span class="badge"><Icon name={t.icon} size={18} /></span>
                <span class="label">{t.label}</span>
              </button>
            {/each}
          </div>
        {/each}
      </div>
      {#if game.buildTool === 'blueprint'}
        <div class="look tall"><Blueprints /></div>
      {:else}
        <div class="look"><LookPicker tool={game.buildTool} /></div>
      {/if}
    </div>
  {/snippet}
</BuildDock>

<style>
  .hint {
    margin: 0;
    min-width: 0;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    animation: step 300ms var(--ease);
  }
  .hint b {
    margin-right: 4px;
    color: var(--build);
  }
  .build {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .tools {
    flex: none;
    display: flex;
    gap: 6px;
  }
  .group {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--hairline);
  }
  .tools button {
    width: 56px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 5px 2px 4px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-weight: 650;
    font-size: 11px;
    transition:
      background var(--fast) var(--ease),
      color var(--fast) var(--ease),
      transform 220ms var(--ease);
  }
  .tools button:hover {
    color: var(--text);
    background: var(--surface);
  }
  .tools button:active {
    transform: scale(0.95);
  }
  .badge {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: 10px;
    background: var(--build-soft);
    color: var(--build);
    transition:
      transform 320ms var(--ease),
      background var(--fast) var(--ease);
  }
  .tools button:hover .badge {
    transform: rotate(-6deg) scale(1.06);
  }
  .tools button.active {
    background: var(--surface);
    color: var(--build);
    box-shadow: var(--shadow-sm);
  }
  .tools button.active .badge {
    background: var(--build);
    color: #fff;
    animation: bounce 420ms var(--ease);
  }
  .tools button:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .label {
    white-space: nowrap;
  }
  .look {
    flex: 1 1 360px;
    min-width: 0;
  }
  .look:empty {
    display: none;
  }
  .look.tall {
    flex-basis: 100%;
    max-height: calc(var(--dock-body) - 80px);
    overflow-y: auto;
  }
  .muted {
    color: var(--text-muted);
  }
  kbd {
    padding: 0 5px;
    border-radius: 4px;
    background: var(--hairline);
    font-family: inherit;
    font-size: 11px;
  }
  @keyframes bounce {
    40% {
      transform: scale(1.18) rotate(-4deg);
    }
  }
  @keyframes step {
    from {
      opacity: 0;
      transform: translateX(-6px);
    }
  }
</style>
