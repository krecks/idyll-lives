<script lang="ts">
  import { streamState } from '../assets/streamState.svelte';

  /** Hidden a little while after the stream finishes; shown again when a new plan starts. */
  let shown = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  const percent = $derived(streamState.total ? Math.round((streamState.done / streamState.total) * 100) : 0);

  $effect(() => {
    if (streamState.active) {
      shown = true;
      clearTimeout(hideTimer);
      return;
    }
    if (streamState.total > 0) {
      hideTimer = setTimeout(() => (shown = false), 4000);
      return () => clearTimeout(hideTimer);
    }
  });
</script>

{#if shown && streamState.total > 0}
  <div class="stream" role="status" title="Models and textures stream in while you play and stay cached for next time">
    <span class="dot" class:done={!streamState.active}></span>
    <span class="text">Game Data is streamed and cached</span>
    <span class="pct">{percent}%</span>
    <span class="bar" aria-hidden="true"><span style:transform={`scaleX(${percent / 100})`}></span></span>
  </div>
{/if}

<style>
  .stream {
    position: absolute;
    left: 50%;
    bottom: calc(var(--edge) + 42px);
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px 8px;
    border-radius: var(--radius-pill);
    background: rgba(29, 34, 48, 0.72);
    color: var(--text-inverse);
    font-size: 11px;
    font-weight: 550;
    letter-spacing: 0.01em;
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(8px);
    pointer-events: none;
    animation: fade-in var(--slow) var(--ease);
  }
  .dot {
    width: 6px;
    height: 6px;
    flex: none;
    border-radius: 50%;
    background: #8fb7ff;
    animation: pulse 1.2s ease-in-out infinite;
  }
  .dot.done {
    background: #8fe0a8;
    animation: none;
  }
  .pct {
    font-variant-numeric: tabular-nums;
    opacity: 0.85;
  }
  /* A hairline under the pill: how much of the plan is cached. */
  .bar {
    position: absolute;
    left: 12px;
    right: 12px;
    bottom: 3px;
    height: 1px;
    overflow: hidden;
    border-radius: 1px;
    background: rgba(255, 255, 255, 0.18);
  }
  .bar span {
    display: block;
    height: 100%;
    background: #8fb7ff;
    transform-origin: left;
    transition: transform var(--slow) var(--ease);
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translate(-50%, 4px);
    }
  }
</style>
