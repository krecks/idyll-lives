// First: errors are captured for debug reports from the start (see `debug/capture.ts`).
import './debug/capture';
import '@fontsource-variable/inter';
import './ui/theme.css';
import { mount } from 'svelte';
import App from './App.svelte';
import { suppressContextMenu } from './ui/contextMenu';
// The voice model and the language model, while they're on (`voice/service.svelte.ts`,
// `talk/talk.svelte.ts`): they load once the game has booted (`ml/schedule.svelte.ts`), not
// beside its first downloads.
import './voice/service.svelte';
import './talk';
import { startModelsAfter } from './ml/schedule.svelte';
import { app } from './ui/app.svelte';
import { watchForDevLog } from './debug/devlogWatch.svelte';

// The dev play log (dev server only, `debug/devlog.ts`).
watchForDevLog();
// No browser right-click menu over the game, except in text fields.
suppressContextMenu();
mount(App, { target: document.getElementById('app')! });
startModelsAfter(() => app.booting);
