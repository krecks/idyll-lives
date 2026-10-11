<script lang="ts">
  import BuildTabs from './buy/BuildTabs.svelte';
  import BuildTools from './buy/BuildTools.svelte';
  import { game } from './state.svelte';

  /** Build mode's Build tab: walls, rooms, doors and windows on the home lot (furniture is the Furnish tab's). */
</script>

{#if game.mode === 'build' && game.buildTab === 'build'}
  <section class="build glass" aria-label="Build">
    <header><BuildTabs /></header>
    <BuildTools />
  </section>
{/if}

<style>
  .build {
    --build: #c9772b;
    --build-soft: rgba(201, 119, 43, 0.14);
    position: absolute;
    left: 50%;
    bottom: var(--edge);
    transform: translateX(-50%);
    width: min(1180px, calc(100vw - 2 * var(--edge)));
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: auto;
  }
  header {
    display: flex;
  }
</style>
