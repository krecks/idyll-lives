/**
 * Imported first (main.ts): the log capture is in place before any other module keeps its own
 * reference to `console.error` (Babylon's logger does, as it loads).
 */

import { installLogCapture } from './log';

installLogCapture();
