// A stand-in `ResidentView` for the residents' tests: it draws nothing, and a test passes what it records.
import { MAX_CHARACTERS } from '../../src/core/snapshot';
import type { ResidentProp, ResidentView } from '../../src/render/shared/characters/view';
import { TemplateCache } from '../../src/render/shared/templateCache';

/** Every call a no-op and every prop a bare one (no matrices drawn), but what `over` gives. */
export function standInView<M extends object = object, Q = object, P extends ResidentProp = ResidentProp>(over: Partial<ResidentView<M, Q, P>> = {}): ResidentView<M, Q, P> {
  return {
    init: () => {},
    setMorphs: () => {},
    uploadPoses: () => {},
    uploadPoseRows: () => {},
    createPart: () => ({}) as M,
    partEnabled: () => {},
    setPartRows: () => {},
    partRows: () => {},
    quiltMaterial: () => {},
    createQuilt: () => ({}) as Q,
    quiltRows: () => {},
    quiltEnabled: () => {},
    createRing: () => {},
    ringMoved: () => {},
    props: new TemplateCache<P | null>({ retell: () => {}, gone: () => false, hide: () => {}, dispose: () => {} }),
    async buildProp(key) {
      return { key, matrices: new Float32Array(MAX_CHARACTERS * 16), top: [0, 0, 0] as const } as unknown as P;
    },
    showProp: () => {},
    onModelFile: () => () => {},
    fileColour: async () => undefined,
    camera: () => false,
    onDispose: () => {},
    ...over,
  };
}
