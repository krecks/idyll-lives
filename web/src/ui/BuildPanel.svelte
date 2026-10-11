<script lang="ts">
  import BuildTools from './buy/BuildTools.svelte';
  import { game } from './state.svelte';

  /** Build mode's Construct tab: walls, rooms, doors and windows on the home lot (furniture is the Furnish tab's). */
</script>

{#if game.mode === 'build' && game.buildTab === 'build'}
  <BuildTools />
{/if}
