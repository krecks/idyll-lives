<script lang="ts">
  import type { HouseholdDraft } from '../game/household';
  import CreateHousehold from './screens/CreateHousehold.svelte';
  import { services } from './services';
  import { game } from './state.svelte';

  /**
   * Create mode: the new game's Creation, for a home nobody lives in (an older save that built
   * first, or a household that's gone): a family moves in. The household creator covers the paused game.
   */
  const neighbours = game.households.filter((h) => !h.player).map((h) => h.name);
  const address = game.plots[game.households.find((h) => h.player)?.plot ?? -1]?.name;

  function moveIn(household: HouseholdDraft) {
    services.controls.moveIn(household);
  }
</script>

<div class="creation">
  <CreateHousehold
    eyebrow={address ? `Creation · moving in · ${address}` : 'Creation · moving in'}
    backLabel="Not yet"
    nextLabel="Move in →"
    avoidNames={neighbours}
    draft={null}
    onback={() => services.controls.setMode('live')}
    onnext={moveIn}
  />
</div>

<style>
  /* Over the game (and its interface), which shows dimmed behind the creator. */
  .creation {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: linear-gradient(180deg, rgba(243, 230, 210, 0.82), rgba(217, 203, 184, 0.9));
    backdrop-filter: blur(6px);
  }
</style>
