/**
 * Things residents ride (content `vehicle`, sim-core `ride.rs`: a bike, an e-scooter) as the renderer draws
 * them under their riders (`residents.ts`): where on it the rider is, its grips, its wheels and its cranks, by
 * object id, measured from its model (tools/art/blender/families/yard.md). The model's own frame: x along it
 * (+x towards its front wheel), y up, z across; its parts are the file's named nodes (`frame`, `wheel_front`,
 * `wheel_rear`, `cranks`, `stand`), each with its origin at its pivot, turned here by code (no clips).
 */

/** A thing to ride, as drawn ridden. */
export interface Mount {
  kind: 'bike' | 'scooter';
  /** The rider's clips (`clips/rides.ts`): sat on a grown-up's bike or a child's, or stood on an e-scooter. */
  clip: 'rideBike' | 'rideBikeChild' | 'rideScooter';
  /** Where the rider is over it: a bike's saddle top where they sit, an e-scooter's deck between the feet (x, height). */
  seat: readonly [number, number];
  /** The grips' middles: x, height, and how far each is out from the middle (z). */
  grips: readonly [number, number, number];
  /** The nodes drawn while it's ridden that don't turn (its frame; not its stand, folded away). */
  still: readonly string[];
  /** Its wheels: the node, its axle (x, height) and its radius. */
  wheels: readonly { node: string; x: number; h: number; r: number }[];
  /**
   * A bike's cranks: the node, its axle (x, height), how long a crank is (axle to pedal), how far out from the
   * middle each pedal is, how many times round the back wheel goes for each turn of them (the gear), and how far
   * forward of straight down the rider's right pedal (on the model's -z side: facing +x, +z is to the left) is in
   * the model (rad; back is negative).
   */
  cranks?: { node: string; x: number; h: number; len: number; out: number; gear: number; rest: number };
}

const WHEELS = (front: [number, number, number], rear: [number, number, number]) => [
  { node: 'wheel_front', x: front[0], h: front[1], r: front[2] },
  { node: 'wheel_rear', x: rear[0], h: rear[1], r: rear[2] },
];

export const MOUNTS: Record<string, Mount> = {
  bicycle: {
    kind: 'bike',
    clip: 'rideBike',
    seat: [-0.3, 0.85],
    grips: [0.2, 0.975, 0.272],
    still: ['frame'],
    wheels: WHEELS([0.55, 0.335, 0.335], [-0.55, 0.335, 0.335]),
    cranks: { node: 'cranks', x: -0.07, h: 0.29, len: 0.17, out: 0.12, gear: 2.2, rest: -Math.PI / 2 },
  },
  bicycleChild: {
    kind: 'bike',
    clip: 'rideBikeChild',
    seat: [-0.17, 0.52],
    grips: [0.16, 0.66, 0.2],
    still: ['frame'],
    wheels: WHEELS([0.28, 0.18, 0.18], [-0.28, 0.18, 0.18]),
    cranks: { node: 'cranks', x: -0.05, h: 0.19, len: 0.105, out: 0.09, gear: 1.8, rest: -Math.PI / 2 },
  },
  eScooter: {
    kind: 'scooter',
    clip: 'rideScooter',
    seat: [-0.01, 0.125],
    grips: [0.284, 1.03, 0.23],
    still: ['frame'],
    wheels: WHEELS([0.37, 0.105, 0.105], [-0.36, 0.105, 0.105]),
  },
};

/** Every node drawn of a ridden `mount`, the still ones first. */
export function mountNodes(mount: Mount): string[] {
  return [...mount.still, ...mount.wheels.map((w) => w.node), ...(mount.cranks ? [mount.cranks.node] : [])];
}

/**
 * Getting on and off (s): the thing slides from where it stands to under its rider, or back, and the rider's
 * clip fades over as long.
 */
export const MOUNT_SECONDS = 0.45;
/** Pedalling no faster than this (turns a second): sped-up games would only blur the legs. */
export const MOST_CADENCE = 1.6;
