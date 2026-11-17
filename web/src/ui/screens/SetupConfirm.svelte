<script lang="ts">
  /**
   * A choice in the new game's steps that would throw the player's own work away (their shaping of
   * the land, their changes to the town), asked first where it was made: what goes, a button to
   * keep it, and one to go ahead.
   */
  let { text, yes, no, label, onanswer }: { text: string; yes: string; no: string; label: string; onanswer: (yes: boolean) => void } = $props();
</script>

<div class="confirm menu-glass" role="alertdialog" aria-label={label}>
  <p>{text}</p>
  <div class="row">
    <button class="btn small" onclick={() => onanswer(false)}>{no}</button>
    <button class="btn small danger" onclick={() => onanswer(true)}>{yes}</button>
  </div>
</div>

<style>
  .confirm {
    margin: 0 var(--pad, 18px) 10px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 550;
    color: var(--text-strong);
    animation: rise 220ms var(--ease) both;
  }
  p {
    margin: 0;
    line-height: 1.4;
  }
  .row {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
  .btn.small {
    height: 30px;
    padding: 0 11px;
    font-size: 12.5px;
  }
  .danger {
    background: var(--bad);
    color: #fff;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
