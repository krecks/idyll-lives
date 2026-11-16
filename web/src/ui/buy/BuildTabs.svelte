<script lang="ts">
  import Icon from '../Icon.svelte';
  import { services } from '../services';
  import { game, type BuildTab } from '../state.svelte';

  /** Build mode's tabs, at the head of its dock: the catalog, the house itself, how the home is doing, and the world's land. */
  const tabs: { id: BuildTab; icon: string; label: string; title: string }[] = [
    { id: 'furnish', icon: 'icon.ui.buy', label: 'Furnish', title: 'Furnish (V): buy, move, upgrade and sell furniture' },
    { id: 'build', icon: 'icon.ui.build', label: 'Construct', title: 'Construct (C): walls, rooms, fences, gates, paint, floors, doors, windows, stairs, the roof and blueprints' },
    { id: 'home', icon: 'icon.ui.home', label: 'Home', title: "Home (O): the rooms and how they're doing, what's broken, and the family" },
    { id: 'land', icon: '', label: 'Land', title: 'Land (G): shape the world: raise and lower the ground, make ponds and rivers, plant woods, raise mountains' },
  ];
</script>

<div class="tabs" role="tablist" aria-label="Build mode">
  {#each tabs as t (t.id)}
    <button role="tab" aria-selected={game.buildTab === t.id} class:active={game.buildTab === t.id} title={t.title} onclick={() => services.controls.setBuildTab(t.id)}>
      {#if t.icon}
        <Icon name={t.icon} size={15} />
      {:else}
        <!-- (No icon of its own in the set: hills under the sun.) -->
        <svg viewBox="0 0 24 24" width="15" height="15" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linejoin="round" stroke-linecap="round">
          <path d="M1.5 20.5 8.5 9l4 6 2.5-3.5 7.5 9zM17.5 7.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z" />
        </svg>
      {/if}
      <span class="label">{t.label}</span>
    </button>
  {/each}
</div>

<style>
  .tabs {
    flex: none;
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--hairline);
  }
  button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 13px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 650;
    color: var(--text-muted);
    white-space: nowrap;
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  button:hover {
    color: var(--text);
  }
  button.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow-sm);
  }
  /* A narrow dock (a small window or a large interface scale): icons only, so the row beside them
     (search) still fits. */
  @container dock (max-width: 820px) {
    button {
      padding: 0 9px;
    }
    .label {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
      white-space: nowrap;
    }
  }
</style>
