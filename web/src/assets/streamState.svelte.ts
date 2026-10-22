/** The stream plan's progress, as the HUD's small indicator reads it. */

import { onStreamChange, streamSnapshot, type StreamSnapshot } from './stream';

class StreamState {
  total = $state(0);
  done = $state(0);
  label = $state('');
  active = $state(false);

  apply(s: StreamSnapshot): void {
    this.total = s.total;
    this.done = s.done;
    this.label = s.label;
    this.active = s.active;
  }
}

export const streamState = new StreamState();

onStreamChange(() => streamState.apply(streamSnapshot()));
streamState.apply(streamSnapshot());
