<script lang="ts">
  import { placeAt } from './place';
  import { game } from './state.svelte';
  import { townMap } from './townMap.svelte';

  /**
   * Live: where the camera is, under the top bar: the lot it looks at and the hamlet round it (a big
   * world's areas), from what the town map knows. Looked up a few times a second; a lot is named
   * as the map names it (a household's home, a public place, an empty lot).
   */
  let text = $state('');
  $effect(() => {
    if (game.mode !== 'live') return;
    const poll = () => {
      const at = townMap.source.camera?.();
      text = at ? placeAt(at[0], at[1], game.plots, game.households, townMap.world?.meta?.areas ?? []) : '';
    };
    poll();
    const timer = setInterval(poll, 300);
    return () => clearInterval(timer);
  });
</script>

{#if game.mode === 'live' && text}
  <div class="place">{text}</div>
{/if}

<style>
  .place {
    position: absolute;
    top: calc(var(--top) - 12px);
    left: 50%;
    transform: translateX(-50%);
    max-width: min(420px, calc(100% - 2 * var(--edge)));
    padding: 4px 12px 5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    border-radius: var(--radius-pill);
    background: var(--glass);
    border: 1px solid var(--glass-border);
    box-shadow: var(--shadow-sm);
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 650;
    pointer-events: none;
  }
</style>
