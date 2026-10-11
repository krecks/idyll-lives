<script lang="ts">
  import Icon from '../Icon.svelte';
  import { services } from '../services';
  import { game, type BuildTab } from '../state.svelte';

  /** Build mode's tabs, at the head of each of its panels: the catalog, the house itself, and how the home is doing. */
  const tabs: { id: BuildTab; icon: string; label: string; title: string }[] = [
    { id: 'furnish', icon: 'icon.ui.buy', label: 'Furnish', title: 'Furnish (V): buy, move, resize, upgrade and sell furniture' },
    { id: 'build', icon: 'icon.ui.build', label: 'Build', title: 'Build: walls, rooms, paint, floors, doors, windows, stairs and the roof' },
    { id: 'home', icon: 'icon.ui.home', label: 'Home', title: "Home (O): the rooms and how they're doing, what's broken, and the family" },
  ];
</script>

<div class="tabs" role="tablist" aria-label="Build mode">
  {#each tabs as t (t.id)}
    <button role="tab" aria-selected={game.buildTab === t.id} class:active={game.buildTab === t.id} title={t.title} onclick={() => services.controls.setBuildTab(t.id)}>
      <Icon name={t.icon} size={15} /><span>{t.label}</span>
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
    height: 28px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    font-size: 12.5px;
    font-weight: 650;
    color: var(--text-muted);
    white-space: nowrap;
    transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  }
  button:hover {
    color: var(--text);
  }
  button.active {
    background: var(--accent);
    color: #fff;
    box-shadow: var(--shadow-sm);
  }
</style>
