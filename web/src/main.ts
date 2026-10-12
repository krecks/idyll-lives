import '@fontsource-variable/inter';
import './ui/theme.css';
import { mount } from 'svelte';
import App from './App.svelte';
import { installLogCapture } from './debug/log';
// The voice model and the language model load as the game launches, while they're on
// (`voice/service.svelte.ts`, `talk/talk.svelte.ts`).
import './voice/service.svelte';
import './talk';

installLogCapture();
mount(App, { target: document.getElementById('app')! });
