<script lang="ts" module>
  /** The page last open, for the next time Settings opens (this visit to the page). */
  let lastPage = 'performance';
</script>

<script lang="ts">
  import { untrack } from 'svelte';
  import { GRAPHICS_PRESETS, graphicsPreset, resetSettings, RESTART_KEYS, settings, type GraphicsPreset, type Settings } from '../../settings/settings.svelte';
  import { app } from '../app.svelte';
  import { services } from '../services';
  import { game } from '../state.svelte';
  import { lifespanHint } from '../format';
  import Icon from '../Icon.svelte';
  import Segmented from '../kit/Segmented.svelte';
  import Toggle from '../kit/Toggle.svelte';
  import Busy from '../kit/Busy.svelte';
  import SettingRow from './SettingRow.svelte';
  import type { Lifespan } from '../../core/protocol';
  import { benchVerdict, DOWNLOAD_MB, runBenchmark, speak, voiceStatus } from '../../voice/service.svelte';
  import { playClip } from '../../voice/player';
  import { voiceFor } from '../../voice/voices';
  import { addModel, allModels, downloadMb, llmStatus, removeModel } from '../../llm';
  import { hasWebGpu } from '../../ml/ort';
  import { currentModel, deviceTest, reportFor, reportKey, sampleLine, stopDeviceTest, testDevices } from '../../talk';

  // This game's lifespan (sent to the simulation when picked).
  let lifespan = $state<Lifespan>(game.lifespan);
  let playerMoves = $state(game.playerMoves);
  $effect(() => {
    if (app.screen === 'game' && lifespan !== game.lifespan) services.controls.setLifespan(lifespan);
  });
  $effect(() => {
    if (app.screen === 'game' && playerMoves !== game.playerMoves) services.controls.setPlayerMoves(playerMoves);
  });

  /** Each page, and the settings "Reset this page" puts back. */
  const pages = [
    { id: 'performance', label: 'Performance', icon: 'icon.ui.perf', keys: ['resolutionScale', 'fpsCap', 'antiAliasing', 'shadowDetail', 'allLamps', 'mirrors', 'ambientOcclusion', 'renderer'] },
    { id: 'gameplay', label: 'Gameplay', icon: 'icon.ui.live', keys: ['autonomy', 'autosaveMinutes'] },
    { id: 'camera', label: 'Camera & time', icon: 'icon.ui.watch', keys: ['directorDelay', 'directorPace', 'hideHudWhileWatching', 'autoSlow', 'skipQuietHours'] },
    { id: 'interface', label: 'Interface', icon: 'icon.ui.display', keys: ['uiScale', 'catalogSize', 'clock24h', 'reducedMotion', 'showFps'] },
    { id: 'audio', label: 'Audio', icon: 'icon.ui.audio', keys: ['sound', 'worldSound', 'worldVolume'] },
    { id: 'voices', label: 'Voices', icon: 'icon.ui.voice', keys: ['voices', 'voiceVolume', 'voiceWriter', 'llmModel', 'subtitles'] },
    { id: 'controls', label: 'Controls', icon: 'icon.ui.move', keys: ['directControl', 'cameraSensitivity'] },
  ] as const satisfies readonly { id: string; label: string; icon: string; keys: readonly (keyof Settings)[] }[];
  type Page = (typeof pages)[number]['id'];
  let page = $state<Page>(pages.some((p) => p.id === lastPage) ? (lastPage as Page) : 'performance');
  const current = $derived(pages.find((p) => p.id === page)!);
  /** "Reset this page" asks first. */
  let confirmReset = $state(false);
  function pickPage(id: Page) {
    page = lastPage = id;
    confirmReset = false;
  }
  /** ↑/↓ between the pages (they're tabs, in a column). */
  function onTabsKey(e: KeyboardEvent) {
    const step = e.key === 'ArrowDown' ? 1 : e.key === 'ArrowUp' ? -1 : 0;
    if (!step) return;
    e.preventDefault();
    const at = pages.findIndex((p) => p.id === page);
    const next = (at + step + pages.length) % pages.length;
    pickPage(pages[next].id);
    ((e.currentTarget as HTMLElement).querySelectorAll('button')[next] as HTMLElement | undefined)?.focus();
  }

  // Performance: a preset, or "Custom" once anything is changed by hand (the rows under Fine-tune).
  const preset = $derived(graphicsPreset(settings));
  let fineTune = $state(graphicsPreset(settings) === 'custom');
  function pickPreset(id: GraphicsPreset | 'custom') {
    if (id === 'custom') return void (fineTune = true);
    Object.assign(settings, GRAPHICS_PRESETS[id]);
  }

  // Remember restart-only values when opened during a game, to show a notice if they change.
  const initial = Object.fromEntries(RESTART_KEYS.map((k) => [k, settings[k]]));
  const needsRestart = $derived(app.screen === 'game' && RESTART_KEYS.some((k) => settings[k] !== initial[k]));
  /** Badge for settings that apply the next time a game starts. */
  const NEXT_LOAD = 'Next load';


  const modelStatus = $derived.by(() => {
    if (voiceStatus.state === 'loading') return `Downloading… ${Math.round(voiceStatus.progress * 100)} %`;
    if (voiceStatus.state === 'error') return `Couldn't load the voice: ${voiceStatus.error}`;
    return `Kyutai's Pocket TTS, on this computer: about ${Math.round(DOWNLOAD_MB)} MB, downloaded once.`;
  });
  const VERDICT = { good: 'smooth', ok: 'lines may start a moment late', slow: 'too slow here: voices may pause' };
  const benchText = $derived.by(() => {
    if (voiceStatus.benchmarking) return 'Testing…';
    const b = voiceStatus.bench;
    if (!b) return 'How fast this computer makes speech while the game keeps drawing.';
    return `${(b.lineMs / 1000).toFixed(1)} s per line, ${(1 / b.rtf).toFixed(1)}× real time, ${VERDICT[benchVerdict(b)]}`;
  });
  let sampleBusy = $state(false);
  let sampleCount = 0;
  async function sample() {
    sampleBusy = true;
    try {
      const male = sampleCount++ % 2 === 1;
      const text = male ? "Hey! I'm one of your residents. Is this what I sound like?" : "Hi! I'm one of your residents. This is how I sound.";
      const voice = voiceFor(sampleCount, male ? 'male' : 'female');
      playClip((await speak(text, voice)).samples);
    } finally {
      sampleBusy = false;
    }
  }

  // The language model that writes lines.
  let gpuAvailable = $state(true);
  void hasWebGpu().then((ok) => (gpuAvailable = ok));
  const model = $derived(currentModel());
  /** The chosen model is downloading (or loading). */
  const downloading = $derived(llmStatus.state === 'loading');
  /** What loading shows: the download, then starting the model (the files are in). */
  const loadLabel = $derived(llmStatus.progress < 1 ? `Downloading ${model.label}` : `Starting ${model.label}`);
  const models = $derived(allModels());
  const mb = (m: (typeof models)[number]) => Math.round(downloadMb(m));
  const llmText = $derived.by(() => {
    if (llmStatus.state === 'error') return `Couldn't load it: ${llmStatus.error}`;
    return `About ${mb(model)} MB, downloaded once.`;
  });
  const NAMES = { A: 'the GPU, voices on the CPU', B: 'the GPU, voices too', C: 'the CPU' };
  const deviceText = $derived.by(() => {
    if (deviceTest.running) return deviceTest.step || 'Testing…';
    const r = reportFor(model);
    if (!r && deviceTest.stopped.includes(reportKey(model))) return 'Test stopped. Until it has run, residents say hand-written lines.';
    if (!r) return gpuAvailable ? 'Not tested here yet: it tests itself when chosen.' : 'No WebGPU here: the model would run on the CPU, which is usually too slow. It tests itself when chosen.';
    const ran = r.results.filter((x) => x.ran);
    if (!r.pick) {
      const why = ran.length ? `${ran.map((x) => `${(x.lineMs / 1000).toFixed(1)} s a line on ${NAMES[x.id]}${x.ok ? '' : ' (wrong text)'}`).join('; ')}.` : 'It couldn\'t load.';
      return `Doesn't run well enough here, so residents say hand-written lines. ${why}`;
    }
    const best = r.results.find((x) => x.id === r.pick!.setup)!;
    return `Runs on ${NAMES[r.pick.setup]}: ${(best.lineMs / 1000).toFixed(2)} s from asking for a line to hearing it.`;
  });
  let writing = $state(false);
  let written = $state('');
  async function writeSample() {
    writing = true;
    try {
      const line = await sampleLine();
      written = line ?? '(nothing usable this time: try again)';
      if (line) playClip((await speak(line, voiceFor(7, 'female'))).samples);
    } catch (e) {
      written = `Failed: ${(e as Error).message}`;
    } finally {
      writing = false;
    }
  }
  let repo = $state('');
  let adding = $state(false);
  let addText = $state('');
  async function add() {
    adding = true;
    addText = 'Looking it up…';
    try {
      const spec = await addModel(repo);
      settings.llmModel = spec.id;
      addText = `Added ${spec.label} (${mb(spec)} MB). It's tested before residents use it.`;
      repo = '';
    } catch (e) {
      addText = (e as Error).message;
    } finally {
      adding = false;
    }
  }
  function remove() {
    const id = model.id;
    settings.llmModel = models[0].id;
    removeModel(id);
  }

  const percent = (v: number) => `${Math.round(v * 100)}%`;

  // Frame rate cap: a preset, or a number of the player's own (`custom`, -1 in the control).
  const FPS_PRESETS = [30, 60, 120, 0];
  const FPS_MIN = 10;
  const FPS_MAX = 500;
  let fpsChoice = $state(FPS_PRESETS.includes(settings.fpsCap) ? settings.fpsCap : -1);
  let customFps = $state(FPS_PRESETS.includes(settings.fpsCap) ? 90 : settings.fpsCap);
  $effect(() => {
    if (fpsChoice !== -1) settings.fpsCap = fpsChoice;
    else if (Number.isFinite(customFps) && customFps > 0) settings.fpsCap = Math.round(Math.min(FPS_MAX, Math.max(FPS_MIN, customFps)));
  });
  // Changed elsewhere (Reset to defaults): the control follows.
  $effect(() => {
    const cap = settings.fpsCap;
    untrack(() => {
      const shown = fpsChoice === -1 ? Math.round(Math.min(FPS_MAX, Math.max(FPS_MIN, customFps))) : fpsChoice;
      if (shown === cap) return;
      fpsChoice = FPS_PRESETS.includes(cap) ? cap : -1;
      if (fpsChoice === -1) customFps = cap;
    });
  });

  /** Shortcuts by area: keys (each a keycap), and what they do. */
  const shortcuts: { title: string; keys: [string[], string][] }[] = [
    {
      title: 'Time',
      keys: [
        [['Space'], 'Pause / resume'],
        [['1', '–', '5'], 'Game speed'],
        [['0'], 'Pause'],
      ],
    },
    {
      title: 'Residents',
      keys: [
        [['Click'], 'Look at a resident, use an object, walk somewhere'],
        [['Tab'], 'Next resident'],
        [['F'], 'Follow the resident'],
      ],
    },
    {
      title: 'Camera',
      keys: [
        [['Drag'], 'Rotate (left) / pan (right)'],
        [['Wheel'], 'Zoom'],
        [['H'], 'Whole house'],
        [['W'], 'Walls up / cutaway / down'],
      ],
    },
    {
      title: 'Modes and panels',
      keys: [
        [['L'], 'Live'],
        [['B'], 'Build'],
        [['V'], 'Build: Furnish'],
        [['C'], 'Build: Construct (walls, rooms, paint)'],
        [['O'], 'Build: Home (how the rooms are doing)'],
        [['P'], 'Planner'],
        [['J'], 'Journal'],
        [['M'], 'Town map'],
        [['Esc'], 'Close / step back / pause menu'],
      ],
    },
    {
      title: 'Build',
      keys: [
        [['R'], 'Rotate (Shift: the other way)'],
        [['E'], 'Eyedropper'],
        [['Del'], 'Sell'],
        [['Ctrl', 'Z'], 'Undo'],
        [['Ctrl', 'Y'], 'Redo'],
      ],
    },
    {
      title: 'Other',
      keys: [
        [['F3'], 'Performance overlay'],
        [['F8'], 'Save a debug report'],
      ],
    },
  ];
</script>

<div class="settings">
  <nav aria-label="Settings">
    <ul role="tablist" aria-orientation="vertical" onkeydown={onTabsKey}>
      {#each pages as p (p.id)}
        <li>
          <button role="tab" aria-selected={page === p.id} tabindex={page === p.id ? 0 : -1} class:active={page === p.id} onclick={() => pickPage(p.id)}>
            <Icon name={p.icon} size={16} />
            {p.label}
          </button>
        </li>
      {/each}
    </ul>
    {#if confirmReset}
      <div class="confirm" role="alert">
        <span>Put {current.label} back to the defaults?</span>
        <div class="row">
          <button class="btn danger small" onclick={() => (resetSettings(current.keys), (confirmReset = false))}>Reset</button>
          <button class="btn ghost small" onclick={() => (confirmReset = false)}>Keep</button>
        </div>
      </div>
    {:else}
      <button class="reset" onclick={() => (confirmReset = true)}>Reset this page</button>
    {/if}
  </nav>

  <div class="page" role="tabpanel">
    {#if needsRestart}
      <p class="notice">Some changes apply the next time you start or load a game.</p>
    {/if}

    {#if page === 'performance'}
      <p class="intro">If the game stutters, pick a lower quality: the resolution saves the most.</p>
      <section>
        <h3>Quality</h3>
        <SettingRow title="Graphics" hint="Resolution, light and shadows, set together." details="Low draws at 75% resolution with simple shadows and only the nearest lamps. High is the default. Fine-tune sets each part on its own.">
          <Segmented
            label="Graphics"
            value={preset}
            options={[
              { value: 'low', label: 'Low' },
              { value: 'medium', label: 'Medium' },
              { value: 'high', label: 'High' },
              { value: 'custom', label: 'Custom' },
            ]}
            onchange={pickPreset}
          />
        </SettingRow>
        <SettingRow title="Frame rate cap" hint="A cap saves power and keeps laptops cooler." details="No cap draws as often as the display refreshes.">
          <Segmented
            label="Frame rate cap"
            bind:value={fpsChoice}
            options={[
              { value: 30, label: '30' },
              { value: 60, label: '60' },
              { value: 120, label: '120' },
              { value: 0, label: 'No cap' },
              { value: -1, label: 'Custom' },
            ]}
          />
          {#if fpsChoice === -1}
            <input class="input fps" type="number" min={FPS_MIN} max={FPS_MAX} step="1" aria-label="Frames per second" bind:value={customFps} />
          {/if}
        </SettingRow>
      </section>
      <button class="disclosure" aria-expanded={fineTune} onclick={() => (fineTune = !fineTune)}>
        <span class="chev" class:open={fineTune}><Icon name="icon.ui.chevron" size={14} /></span>Fine-tune
      </button>
      {#if fineTune}
        <section>
          <h3>Image</h3>
          <SettingRow title="Resolution" hint="Lower is softer, but much faster.">
            <input class="slider" type="range" min="0.5" max="1" step="0.05" aria-label="Resolution scale" bind:value={settings.resolutionScale} />
            <output>{percent(settings.resolutionScale)}</output>
          </SettingRow>
          <SettingRow title="Smooth edges" hint="Removes jagged edges." details="4× multisampling. Off uses a cheaper filter that blurs edges a little." badges={[NEXT_LOAD]}>
            <Toggle label="Smooth edges" bind:checked={settings.antiAliasing} />
          </SettingRow>
        </section>
        <section>
          <h3>Light and shadow</h3>
          <SettingRow title="Shadow detail" hint="Each step up uses four times the graphics memory." badges={[NEXT_LOAD]}>
            <Segmented
              label="Shadow detail"
              bind:value={settings.shadowDetail}
              options={[
                { value: 1024, label: 'Low' },
                { value: 2048, label: 'Medium' },
                { value: 4096, label: 'High' },
              ]}
            />
          </SettingRow>
          <SettingRow title="Every lamp lights" hint="Off: only the four lamps nearest the biggest rooms." details="On: up to 32 lamps light their rooms. Costs little on WebGPU, more on WebGL2." badges={[NEXT_LOAD]}>
            <Toggle label="Every lamp lights" bind:checked={settings.allLamps} />
          </SettingRow>
          <SettingRow title="Mirror reflections" hint="The two mirrors nearest the camera show the room." details="Each draws the scene once more while it is on screen." badges={[NEXT_LOAD]}>
            <Toggle label="Mirror reflections" bind:checked={settings.mirrors} />
          </SettingRow>
          <SettingRow title="Ambient occlusion" hint="Soft contact shadows in corners and under furniture." details="Costly, and may fail to start on some WebGPU drivers." badges={['Experimental', NEXT_LOAD]}>
            <Toggle label="Ambient occlusion" bind:checked={settings.ambientOcclusion} />
          </SettingRow>
        </section>
        <section>
          <h3>Engine</h3>
          <SettingRow title="Graphics API" hint="WebGL2 is the fallback if the game won't draw." details="WebGPU (Auto, where there is one) is faster." badges={[NEXT_LOAD]}>
            <Segmented
              label="Graphics API"
              bind:value={settings.renderer}
              options={[
                { value: 'auto', label: 'Auto' },
                { value: 'webgl', label: 'WebGL2' },
              ]}
            />
          </SettingRow>
        </section>
      {/if}
    {:else if page === 'gameplay'}
      {#if app.screen === 'game'}
        <section>
          <h3>This game</h3>
          <SettingRow title="Lifespan" hint={lifespanHint(lifespan)}>
            <Segmented
              label="Lifespan"
              bind:value={lifespan}
              options={[
                { value: 'off', label: 'Off' },
                { value: 'short', label: 'Short' },
                { value: 'normal', label: 'Normal' },
                { value: 'long', label: 'Long' },
              ]}
            />
          </SettingRow>
          <SettingRow title="Moving" hint="Residents move in with partners and out of home on their own.">
            <Toggle label="Moving" bind:checked={playerMoves} />
          </SettingRow>
        </section>
      {/if}
      <section>
        <h3>Residents</h3>
        <SettingRow title="Free will" hint="Residents look after needs, friends and work themselves." details="Neighbours always do.">
          <Toggle label="Free will" bind:checked={settings.autonomy} />
        </SettingRow>
      </section>
      <section>
        <h3>Saving</h3>
        <SettingRow title="Autosave" hint="Into a single autosave slot.">
          <Segmented
            label="Autosave"
            bind:value={settings.autosaveMinutes}
            options={[
              { value: 0, label: 'Off' },
              { value: 2, label: '2 min' },
              { value: 5, label: '5 min' },
              { value: 10, label: '10 min' },
            ]}
          />
        </SettingRow>
      </section>
    {:else if page === 'camera'}
      <section>
        <h3>Camera in Live</h3>
        <SettingRow title="Back to watching" hint="With no input this long, the camera follows life at home." details="Any input takes it back.">
          <Segmented
            label="Back to watching"
            bind:value={settings.directorDelay}
            options={[
              { value: 0, label: 'Off' },
              { value: 20, label: '20 s' },
              { value: 45, label: '45 s' },
              { value: 90, label: '90 s' },
            ]}
          />
        </SettingRow>
        <SettingRow title="Camera pace" hint="Calm lingers on each moment; lively moves on sooner.">
          <Segmented
            label="Camera pace"
            bind:value={settings.directorPace}
            options={[
              { value: 'calm', label: 'Calm' },
              { value: 'lively', label: 'Lively' },
            ]}
          />
        </SettingRow>
        <SettingRow title="Quiet interface" hint="The interface fades while the camera watches." details="Move the mouse to bring it back.">
          <Toggle label="Quiet interface" bind:checked={settings.hideHudWhileWatching} />
        </SettingRow>
      </section>
      <section>
        <h3>Time</h3>
        <SettingRow title="Slow down for big moments" hint="A first kiss, a new job or a fight at home plays at normal speed.">
          <Toggle label="Slow down for big moments" bind:checked={settings.autoSlow} />
        </SettingRow>
        <SettingRow title="Skip quiet hours" hint="Time-lapse while everyone at home sleeps or is at work.">
          <Toggle label="Skip quiet hours" bind:checked={settings.skipQuietHours} />
        </SettingRow>
      </section>
    {:else if page === 'interface'}
      <section>
        <h3>Display</h3>
        <SettingRow title="Interface size" hint="Scales menus and the HUD.">
          <Segmented
            label="Interface size"
            bind:value={settings.uiScale}
            options={[
              { value: 0.9, label: 'S' },
              { value: 1, label: 'M' },
              { value: 1.1, label: 'L' },
              { value: 1.25, label: 'XL' },
            ]}
          />
        </SettingRow>
        <SettingRow title="Build catalog" hint="One row keeps more of your lot in view." details="The button at the end of the Build dock's header switches it too.">
          <Segmented
            label="Build catalog"
            bind:value={settings.catalogSize}
            options={[
              { value: 'compact', label: 'One row' },
              { value: 'expanded', label: 'Several rows' },
            ]}
          />
        </SettingRow>
        <SettingRow title="First-use tips" hint="Show the tips for each panel again.">
          <button class="btn ghost" disabled={settings.tipsSeen.length === 0} onclick={() => (settings.tipsSeen = [])}>Show again</button>
        </SettingRow>
        <SettingRow title="24-hour clock" hint="Otherwise AM / PM.">
          <Toggle label="24-hour clock" bind:checked={settings.clock24h} />
        </SettingRow>
        <SettingRow title="Reduce motion" hint="Turns off interface animations.">
          <Toggle label="Reduce motion" bind:checked={settings.reducedMotion} />
        </SettingRow>
      </section>
      <section>
        <h3>Diagnostics</h3>
        <SettingRow title="Performance overlay" hint="Frame rate, draw calls and graphics backend (F3).">
          <Toggle label="Performance overlay" bind:checked={settings.showFps} />
        </SettingRow>
      </section>
    {:else if page === 'audio'}
      <section>
        <h3>Sound</h3>
        <SettingRow title="Sound effects" hint="Little sounds when you buy, build and place things.">
          <Toggle label="Sound effects" bind:checked={settings.sound} />
        </SettingRow>
        <SettingRow title="Sounds of the world" hint="Birds by day, crickets at night, a pan sizzling, the TV.">
          <Toggle label="Sounds of the world" bind:checked={settings.worldSound} />
        </SettingRow>
        <SettingRow title="World volume" disabled={!settings.worldSound}>
          <input class="slider" type="range" min="0" max="1" step="0.05" aria-label="World volume" bind:value={settings.worldVolume} disabled={!settings.worldSound} />
          <output>{percent(settings.worldVolume)}</output>
        </SettingRow>
      </section>
    {:else if page === 'voices'}
      <section>
        <h3>Voices</h3>
        <SettingRow
          title="Resident voices"
          hint="Residents speak out loud, in English."
          details="The resident you're looking at, and conversations you start. Runs on this computer; the voices download about {Math.round(DOWNLOAD_MB)} MB once."
          badges={['Experimental']}
        >
          <Toggle label="Resident voices" bind:checked={settings.voices} />
        </SettingRow>
        {#if settings.voices}
          <SettingRow title="Voice volume">
            <input class="slider" type="range" min="0" max="1" step="0.05" aria-label="Voice volume" bind:value={settings.voiceVolume} />
            <output>{percent(settings.voiceVolume)}</output>
          </SettingRow>
          <SettingRow title="Voice model" hint={modelStatus} details="Speaks like a short recording: 24 voices to start from, each resident with their own pitch, depth and speed, children higher and smaller; or a voice you record in the household creator. Pocket TTS is © Kyutai, CC BY 4.0.">
            {#if voiceStatus.state === 'loading'}
              <Busy label="Downloading the voices" progress={voiceStatus.progress} />
            {:else}
              <output>{voiceStatus.state === 'ready' ? 'Ready' : voiceStatus.state === 'error' ? 'Failed' : 'Not loaded'}</output>
            {/if}
          </SettingRow>
          <SettingRow title="Try the voices" hint={benchText}>
            <button class="btn ghost" disabled={sampleBusy} onclick={sample}>{#if sampleBusy}<span class="spinner" aria-hidden="true"></span>{/if}Hear a sample</button>
            <button class="btn ghost" disabled={voiceStatus.benchmarking} onclick={() => void runBenchmark().catch(() => {})}>
              {#if voiceStatus.benchmarking}<span class="spinner" aria-hidden="true"></span>{/if}Speed test
            </button>
          </SettingRow>
        {/if}
      </section>
      {#if settings.voices}
        <section>
          <h3>What residents say</h3>
          <SettingRow
            title="Lines"
            hint="Hand-written, or written by a small language model."
            details="The model runs on this computer and writes each line from who the resident is and what's going on: traits, mood, plans, job, what's around them, the time of day."
            badges={settings.voiceWriter === 'llm' ? ['Experimental'] : []}
          >
            <Segmented
              label="Lines"
              bind:value={settings.voiceWriter}
              options={[
                { value: 'lines', label: 'Hand-written' },
                { value: 'llm', label: 'Language model' },
              ]}
            />
          </SettingRow>
          <SettingRow title="Subtitles" hint="Spoken lines also show in the speech bubble.">
            <Toggle label="Subtitles" bind:checked={settings.subtitles} />
          </SettingRow>
        </section>
        {#if settings.voiceWriter === 'llm'}
          <section>
            <h3>Language model</h3>
            <SettingRow title="Model" hint={llmText} details="{model.about} Licence: {model.licence}.">
              <select class="input" aria-label="Model" bind:value={settings.llmModel} disabled={deviceTest.running}>
                <optgroup label="Built in">
                  {#each models.filter((m) => !m.added) as m (m.id)}<option value={m.id}>{m.label} · {mb(m)} MB</option>{/each}
                </optgroup>
                {#if models.some((m) => m.added)}
                  <optgroup label="Added">
                    {#each models.filter((m) => m.added) as m (m.id)}<option value={m.id}>{m.label} · {mb(m)} MB</option>{/each}
                  </optgroup>
                {/if}
              </select>
            </SettingRow>
            <SettingRow title="On this computer" hint={deviceTest.running ? 'Writes and speaks a few lines where it can run, to find the fastest that works.' : deviceText}>
              {#if deviceTest.running}
                <Busy label={downloading ? loadLabel : deviceTest.step} progress={downloading && llmStatus.progress < 1 ? llmStatus.progress : deviceTest.progress} />
                <button class="btn ghost" disabled={deviceTest.stopping} onclick={stopDeviceTest}>Stop</button>
              {:else}
                <button class="btn ghost" onclick={() => void testDevices().catch(() => {})}>{reportFor(model) ? 'Test again' : 'Test'}</button>
              {/if}
            </SettingRow>
            <SettingRow title="Try the model" hint={written ? `“${written}”` : 'Writes a line for a sample moment and says it.'}>
              {#if writing && downloading}
                <Busy label={loadLabel} progress={llmStatus.progress < 1 ? llmStatus.progress : null} />
              {/if}
              <button class="btn ghost" disabled={writing || deviceTest.running} onclick={writeSample}>
                {#if writing}<span class="spinner" aria-hidden="true"></span>{/if}{writing && !downloading ? 'Writing…' : 'Write a sample line'}
              </button>
            </SettingRow>
            <SettingRow
              title="Add a model"
              hint={addText || 'A small Hugging Face model with a 4-bit ONNX build.'}
              details="It needs onnx/model_q4.onnx, as onnx-community/SmolLM2-135M-Instruct-ONNX has. It downloads from Hugging Face when used."
            >
              <input class="input repo" type="text" placeholder="owner/model" aria-label="Hugging Face model" bind:value={repo} onkeydown={(e) => e.key === 'Enter' && repo.trim() && !adding && void add()} />
              <button class="btn ghost" disabled={adding || !repo.trim()} onclick={add}>{#if adding}<span class="spinner" aria-hidden="true"></span>{/if}Add</button>
            </SettingRow>
            {#if model.added}
              <SettingRow title="Remove {model.label}" hint="Takes it off the list." details="Its download is freed the next time a model loads.">
                <button class="btn ghost" disabled={deviceTest.running} onclick={remove}>Remove</button>
              </SettingRow>
            {/if}
          </section>
        {/if}
      {/if}
    {:else}
      <section>
        <h3>Mouse</h3>
        <SettingRow title="Clicking residents" hint="Look first opens their panel; Give orders commands at once." details="Give orders: clicks command the selected resident straight away.">
          <Segmented
            label="Clicking residents"
            bind:value={settings.directControl}
            options={[
              { value: 'inspect', label: 'Look first' },
              { value: 'always', label: 'Give orders' },
            ]}
          />
        </SettingRow>
        <SettingRow title="Camera sensitivity" hint="Rotate, pan and zoom speed.">
          <input class="slider" type="range" min="0.5" max="2" step="0.1" aria-label="Camera sensitivity" bind:value={settings.cameraSensitivity} />
          <output>{settings.cameraSensitivity.toFixed(1)}×</output>
        </SettingRow>
      </section>
      <section>
        <h3>Keyboard</h3>
        <div class="shortcuts">
          {#each shortcuts as group (group.title)}
            <div class="group">
              <h4>{group.title}</h4>
              <dl>
                {#each group.keys as [keys, action] (action)}
                  <dt>{#each keys as k, i (i)}{#if k === '–'}<span class="to">–</span>{:else}<kbd>{k}</kbd>{/if}{/each}</dt>
                  <dd>{action}</dd>
                {/each}
              </dl>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  </div>
</div>

<style>
  .settings {
    display: grid;
    grid-template-columns: 180px 1fr;
    gap: 24px;
    height: min(560px, calc(100vh / var(--ui-scale) - 140px));
  }
  nav {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 12px;
  }
  nav ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  nav li button {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 38px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    color: var(--text-muted);
    text-align: left;
  }
  nav li button:hover {
    background: var(--hairline);
    color: var(--text);
  }
  nav li button.active {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow-sm);
  }
  .reset {
    height: 34px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-muted);
    text-align: left;
  }
  .reset:hover {
    background: var(--hairline);
    color: var(--bad);
  }
  .confirm {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    font-size: 12.5px;
    font-weight: 600;
  }
  .confirm .row {
    display: flex;
    gap: 4px;
  }
  .confirm .btn {
    height: 28px;
    padding: 0 10px;
    font-size: 12px;
  }
  .disclosure {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px 0 8px;
    border-radius: var(--radius-pill);
    font-size: 12.5px;
    font-weight: 650;
    color: var(--text-muted);
  }
  .disclosure:hover {
    background: var(--hairline);
    color: var(--text);
  }
  .chev {
    display: inline-grid;
    transform: rotate(90deg);
    transition: transform var(--fast) var(--ease);
  }
  .chev.open {
    transform: rotate(180deg);
  }
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
    min-width: 0;
    overflow-y: auto;
    padding-right: 4px;
  }
  section {
    padding: 4px 16px;
    border-radius: var(--radius-md);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
  }
  h3 {
    margin: 12px 0 2px;
    font-size: 11px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }
  .intro {
    margin: 0;
    color: var(--text-muted);
    font-size: 13px;
  }
  .notice {
    margin: 0;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    background: rgba(240, 181, 74, 0.16);
    color: #8a5e0c;
    font-weight: 550;
    font-size: 13px;
  }
  .slider {
    width: 170px;
  }
  select.input {
    width: 280px;
  }
  .repo {
    width: 220px;
  }
  .fps {
    width: 76px;
  }
  output {
    min-width: 40px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    font-size: 13px;
  }
  .shortcuts {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 4px 24px;
    padding: 8px 0 14px;
  }
  h4 {
    margin: 8px 0 6px;
    font-size: 12.5px;
    font-weight: 650;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 6px 12px;
    margin: 0;
  }
  dt {
    display: flex;
    align-items: center;
    gap: 3px;
  }
  dd {
    margin: 0;
    color: var(--text-muted);
    font-size: 12.5px;
  }
  kbd {
    min-width: 22px;
    height: 22px;
    padding: 0 6px;
    display: inline-grid;
    place-items: center;
    border-radius: 6px;
    background: var(--surface-muted);
    border: 1px solid var(--hairline);
    border-bottom-width: 2px;
    font: inherit;
    font-size: 11.5px;
    font-weight: 650;
  }
  .to {
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
