<script lang="ts">
  /**
   * The Village tool's card: a new village laid along or turned across (`villageChoice.swap`), or,
   * with a village being moved (its card's "Move"), which and how to stop. The village being moved
   * is let go when the tool is put down.
   */
  import { onDestroy } from 'svelte';
  import Segmented from '../kit/Segmented.svelte';
  import type { TownEditor } from './tools';
  import { villageChoice } from './villageChoice.svelte';

  const { editor }: { editor: TownEditor } = $props();

  const SHAPES = [
    { value: 'along', label: 'Along' },
    { value: 'across', label: 'Across' },
  ] as const;

  const moving = $derived((void editor.version, villageChoice.moving === null ? undefined : editor.draft.open?.areas[villageChoice.moving]));

  onDestroy(() => (villageChoice.moving = null));
</script>

<div class="village-card">
  {#if moving}
    <p class="hint"><b>Moving {moving.name}.</b> Click where it goes; its streets, homes and buildings go with it, and a road to the town is laid again.</p>
    <div class="row">
      <button class="btn small ghost" onclick={() => (villageChoice.moving = null)}>Stop moving it</button>
    </div>
  {:else}
    <div class="option">
      <span class="eyebrow sub">Its streets</span>
      <Segmented fill label="How the village lies" value={villageChoice.swap ? 'across' : 'along'} options={SHAPES} onchange={(v) => (villageChoice.swap = v === 'across')} />
    </div>
    <p class="hint">A village of this place's own kind: its streets, a road to the town and its homes. A village put down by hand has no town buildings. The main town moves with Move the town.</p>
  {/if}
</div>

<style>
  .village-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .hint {
    margin: 0;
  }
</style>
