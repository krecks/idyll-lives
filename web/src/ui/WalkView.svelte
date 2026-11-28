<script lang="ts">
  /** Walk mode (`game/walk.ts`): the crosshair F aims with, the keys, and the typed chat with a resident (the model answers). */
  import { firstName } from '../talk/context';
  import { services } from './services';
  import { game } from './state.svelte';

  const other = $derived(game.talkWith === null ? '' : firstName(game.talkWith));

  let lines = $state<{ who: string; text: string }[]>([]);
  let words = $state('');
  let waiting = $state(false);
  let ask: AbortController | null = null;

  // Each chat starts afresh; closing it drops an answer still being written.
  $effect(() => {
    void game.talkWith;
    return () => {
      ask?.abort();
      lines = [];
      words = '';
      waiting = false;
    };
  });

  async function send(e: SubmitEvent) {
    e.preventDefault();
    const said = words.trim();
    if (!said || waiting || game.talkWith === null) return;
    const before = lines
      .slice(-8)
      .map((l) => `${l.who}: "${l.text}"`)
      .join('\n');
    const me = game.talkFrom === null ? '' : firstName(game.talkFrom);
    const them = other;
    lines = [...lines, { who: me, text: said }];
    words = '';
    waiting = true;
    const mine = (ask = new AbortController());
    const reply = await services.controls.answer(said, before, mine.signal).catch(() => null);
    if (mine.signal.aborted) return;
    waiting = false;
    lines = [...lines, { who: them, text: reply ?? `(${them} says nothing.)` }];
  }
</script>

{#if game.walking !== null && game.mode === 'live'}
  <div class="cross" aria-hidden="true"></div>
  {#if game.talkWith === null}
    <p class="keys glass">W/S walk (W also stops what they're doing) · Shift run · A/D turn · Q/E step aside · drag to look · F use or talk · Esc leave</p>
  {/if}
{/if}

{#if game.talkWith !== null}
  <div class="chat glass" role="dialog" aria-label="Talk with {other}">
    <header>
      <b>Talking with {other}</b>
      <button class="btn" onclick={() => services.controls.openTalk(null)}>Close</button>
    </header>
    <div class="lines" aria-live="polite">
      {#each lines as l, i (i)}
        <p><b>{l.who}:</b> {l.text}</p>
      {/each}
      {#if waiting}<p class="muted">{other} is thinking…</p>{/if}
    </div>
    <form onsubmit={send}>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        bind:value={words}
        placeholder="Say something…"
        maxlength="200"
        autofocus
        aria-label="What you say"
        onkeydown={(e) => e.key === 'Escape' && services.controls.openTalk(null)}
      />
      <button class="btn" type="submit" disabled={waiting || !words.trim()}>Say</button>
    </form>
  </div>
{/if}

<style>
  .cross {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 6px;
    height: 6px;
    margin: -3px 0 0 -3px;
    border-radius: 50%;
    background: rgb(255 255 255 / 0.85);
    box-shadow: 0 0 0 1px rgb(0 0 0 / 0.4);
  }
  .keys {
    position: absolute;
    left: 50%;
    bottom: 120px;
    transform: translateX(-50%);
    margin: 0;
    padding: 6px 12px;
    font-size: 12px;
  }
  .chat {
    position: absolute;
    right: 16px;
    bottom: 120px;
    width: 340px;
    max-height: 420px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    pointer-events: auto;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .lines {
    overflow-y: auto;
    flex: 1;
    font-size: 13px;
  }
  .lines p {
    margin: 4px 0;
  }
  .muted {
    color: var(--text-muted);
  }
  form {
    display: flex;
    gap: 6px;
  }
  input {
    flex: 1;
  }
</style>
