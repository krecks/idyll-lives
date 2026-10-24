/** Whether WebGPU can be used this session: once it had an adapter but failed to start, later engines (the game's, the previews') use WebGL2 straight away. */

let webGpuBroken = false;

export const webGpuFailed = (): boolean => webGpuBroken;

export function noteWebGpuFailed(): void {
  webGpuBroken = true;
}
