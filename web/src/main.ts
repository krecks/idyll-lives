// First: errors are captured for debug reports from the start (see `debug/capture.ts`).
import './debug/capture';
import '@fontsource-variable/inter';
import './ui/theme.css';
import { mount } from 'svelte';
import App from './App.svelte';
// The voice model and the language model load as the game launches, while they're on
// (`voice/service.svelte.ts`, `talk/talk.svelte.ts`).
import './voice/service.svelte';
import './talk';

mount(App, { target: document.getElementById('app')! });
