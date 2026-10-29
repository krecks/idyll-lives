// A camera following a resident (`keepUp` in render/shared/cameraRig.ts): the player's follow (the renderer's half-second
// easing) and the watching director's shot (`GameCameraRig`, 0.9 s) stay within `FOLLOW_GAP` of a walker once caught
// up, at every game speed; before that, a glide to someone far off eases in rather than jumping.
import { describe, expect, test } from 'bun:test';
import { FOLLOW_GAP, GameCameraRig, keepUp } from '../src/render/shared/cameraRig';

/** A resident's walking speed (m/s) at each game speed (1.5 m/s times `GAME_RATE`). */
const SPEEDS = [1.5, 4.5, 15, 45, 90];
const DT = 1 / 60;

describe('the camera following a resident', () => {
  test("the player's follow: never more than the gap behind once caught up, at any speed", () => {
    for (const v of SPEEDS) {
      const t = { x: 0, z: 0 };
      let caught = false;
      let worst = 0;
      for (let f = 0; f < 120; f++) {
        const x = v * f * DT;
        // (The renderer's easing, then `keepUp`.)
        const k = 1 - Math.exp(-DT / 0.5);
        t.x += (x - t.x) * k;
        caught = keepUp(t, x, 0, caught);
        worst = Math.max(worst, Math.hypot(t.x - x, t.z));
      }
      expect(worst).toBeLessThanOrEqual(FOLLOW_GAP + 1e-9);
    }
  });

  test('a follow started far off glides in rather than jumping', () => {
    const t = { x: 0, z: 0 };
    let caught = false;
    let biggest = 0;
    for (let f = 0; f < 240; f++) {
      const was = t.x;
      t.x += (30 - t.x) * (1 - Math.exp(-DT / 0.5));
      caught = keepUp(t, 30, 0, caught);
      biggest = Math.max(biggest, t.x - was);
    }
    expect(caught).toBe(true);
    // (Eased: under a metre a frame, not 28 m at once.)
    expect(biggest).toBeLessThan(1);
  });

  test("the director's shot following a resident walking at speed 4", () => {
    const head = { x: 0, z: 0 };
    const camera = { alpha: 0, beta: 0.9, radius: 12, target: { x: 0, y: 0, z: 0 } };
    const rig = new GameCameraRig(camera, { radius: [4, 60], beta: [0.3, 1.4] }, (_, out) => {
      out.x = head.x;
      out.y = 1.6;
      out.z = head.z;
      return true;
    }, () => {});
    rig.setShot({ follow: [0], duration: 0.5 });
    let worst = 0;
    for (let f = 0; f < 240; f++) {
      if (f > 60) head.x += 45 * DT;
      rig.frame(DT);
      if (f > 30) worst = Math.max(worst, Math.hypot(camera.target.x - head.x, camera.target.z - head.z));
    }
    expect(worst).toBeLessThanOrEqual(FOLLOW_GAP + 1e-6);
  });
});
