<script lang="ts">
  /**
   * The Lot tool's card: what it puts down (`lotChoice`). A home: a house of a size, with a garden (or as the
   * world's homes have theirs), or a building of flats; a farm; one of the public places a town's lot can be; an outdoor
   * spot; or a civic place the town or village under the pointer hasn't got, put down as its
   * building along a street.
   */
  import { publicChoices } from '../../game/edit/lots';
  import Segmented from '../kit/Segmented.svelte';
  import { civicChosen, civicLacking, lotChoice, outdoorChoices, type HomeKind, type HomeSize, type LotGroup } from './lotChoice.svelte';
  import type { TownEditor } from './tools';

  const { editor }: { editor: TownEditor } = $props();

  const GROUPS: readonly { value: LotGroup; label: string }[] = [
    { value: 'home', label: 'Home' },
    { value: 'farm', label: 'Farm' },
    { value: 'public', label: 'Public' },
    { value: 'outdoor', label: 'Outdoor' },
    { value: 'civic', label: 'Civic' },
  ];
  const KINDS: readonly { value: HomeKind; label: string }[] = [
    { value: 'house', label: 'House' },
    { value: 'flats', label: 'Flats' },
  ];
  const SIZES: readonly { value: HomeSize; label: string }[] = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];
  const GARDENS = [
    { value: 'any', label: 'Any' },
    { value: 'snug', label: 'Snug' },
    { value: 'roomy', label: 'Roomy' },
    { value: 'big', label: 'Big' },
  ] as const;

  const t = $derived(editor.t);
  const nameOf = (id: string) => t.parks.find((p) => p.id === id)?.name ?? id;
  const places = $derived(publicChoices(t));
  const outdoors = $derived(outdoorChoices(t));
  const civics = $derived((void editor.version, civicLacking(editor.draft, lotChoice.civicArea)));
  const civicOn = $derived((void editor.version, civicChosen(editor.draft, lotChoice.civicArea)));
  const civicWhere = $derived(editor.draft.open?.areas[lotChoice.civicArea]?.name ?? editor.townName);
</script>

<div class="lot-card">
  <Segmented fill label="What to put down" bind:value={lotChoice.group} options={GROUPS} />
  {#if lotChoice.group === 'home'}
    <Segmented fill label="Home" bind:value={lotChoice.home} options={KINDS} />
  {/if}
  {#if lotChoice.group === 'home' && lotChoice.home === 'flats'}
    <p class="hint">A building of flats round a stair hall, two or three storeys high (mostly three in a town centre), each flat a home of its own. What it is is said as you point.</p>
  {:else if lotChoice.group === 'home'}
    <div class="option">
      <span class="eyebrow sub">Size</span>
      <Segmented fill label="Home size" bind:value={lotChoice.size} options={SIZES} />
    </div>
    <div class="option">
      <span class="eyebrow sub">Garden</span>
      <Segmented fill label="Garden" bind:value={lotChoice.garden} options={GARDENS} />
    </div>
  {:else if lotChoice.group === 'farm'}
    <p class="hint">A farmhouse with its fields and farmyard. It needs a wide stretch of road, out of the town centre.</p>
  {:else if lotChoice.group === 'public'}
    <div class="choices" role="radiogroup" aria-label="Public place">
      {#each places as id, i (id)}
        <button role="radio" class="chip" aria-checked={(lotChoice.place || places[0]) === id} class:on={(lotChoice.place || places[0]) === id} tabindex={i === 0 ? 0 : -1} onclick={() => (lotChoice.place = id)}>{nameOf(id)}</button>
      {/each}
    </div>
  {:else if lotChoice.group === 'outdoor'}
    <div class="choices" role="radiogroup" aria-label="Outdoor spot">
      {#each outdoors as id, i (id)}
        <button role="radio" class="chip" aria-checked={(lotChoice.outdoor || outdoors[0]) === id} class:on={(lotChoice.outdoor || outdoors[0]) === id} tabindex={i === 0 ? 0 : -1} onclick={() => (lotChoice.outdoor = id)}>{nameOf(id)}</button>
      {/each}
    </div>
    <p class="hint">A short footpath is laid to it from the road.</p>
  {:else}
    {#if civics.length}
      <span class="eyebrow sub">What {civicWhere} hasn't got</span>
      <div class="choices" role="radiogroup" aria-label="Civic places {civicWhere} has not got">
        {#each civics as c, i (c.civic)}
          <button role="radio" class="chip" aria-checked={civicOn === c.civic} class:on={civicOn === c.civic} tabindex={i === 0 ? 0 : -1} onclick={() => (lotChoice.civic = c.civic)}>{c.label}</button>
        {/each}
      </div>
      <p class="hint">Put down along a street, facing it. The list follows the town or village under the pointer.</p>
    {:else}
      <p class="hint">{civicWhere} has every civic place already.</p>
    {/if}
  {/if}
</div>

<style>
  .lot-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .option {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    height: 28px;
    padding: 0 10px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    color: var(--text);
    font-size: 12px;
    font-weight: 600;
  }
  .chip.on {
    background: var(--accent);
    color: #fff;
  }
  .chip:disabled {
    opacity: 0.55;
    cursor: default;
  }
  .hint {
    margin: 0;
  }
</style>
