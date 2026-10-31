/**
 * The look's light strengths (`render/look.ts`) in PlayCanvas's units: one factor per kind of
 * light, each with its reason.
 *
 * The look keeps the units it was tuned in (Babylon.js's PBR, the game's renderer up to 0.3.2): a
 * directional light, or a lamp at a metre, of strength 1 falling square on lights a white surface
 * to 1 / π (a Lambert surface's `albedo / π` times the light it receives); the sky's hemispheric
 * fill of strength 1 lights it to 1; the image-based light's strength scales the sky's own
 * irradiance and reflections. PlayCanvas lights a white surface to a direct light's strength
 * itself, so the factors below convert.
 *
 * Calibrated against pictures of the game as it looked up to 0.3.2 (during the port: the bench
 * poses with `?diag=linear,nosims`, the lit colour before any image processing, and
 * `?lightmix=sun,sky,env` lighting one part at a time, at 6:36, 9, 13, 16, 18:30, 19:30, 20:12 and
 * 23:00), after the parts of the light that were not a matter of units were matched to those
 * pictures first: the sky's ambient harmonics (`cargo envconv`), the haze (`shaders/fog.ts`), the AO
 * map on the direct light (`shaders/softLook.ts`) and the reflection's rise at a slant
 * (`shaders/envBrdf.ts`).
 */

/**
 * The sun (and the moon): in the look's units a directional light of strength `I` lights a surface
 * to `albedo / π · I` (Lambert), PlayCanvas to `albedo · I`: the look's strengths / π. (The lamps,
 * `lamps.ts` `LAMP_UNITS`, are in the same units.)
 */
export const SUN_UNITS = 1 / Math.PI;

/**
 * The sky's hemispheric fill (`setSkyFill`, the soft look's `gameHemi`): in the look's units a
 * hemispheric light has no `1 / π` (a white surface facing the sky takes the sky's colour times the
 * strength), and neither has the game's term: the look's strengths as they are. (Its light alone at
 * noon: within 0.3 of 255 of the earlier pictures on average.)
 */
export const SKY_UNITS = 1;

/**
 * Image-based light (`scene.skyboxIntensity`: the sky's ambient harmonics and reflections): the
 * look's environment strengths, 5 % up. The harmonics are the irradiance the look was tuned with,
 * exactly, but PlayCanvas takes the specularity off the ambient once more (`dDiffuseLight * (1 −
 * F0)`, on top of the soft look's own `DIELECTRIC`), where the tuning took off only the environment
 * BRDF's reflectance, which is less at most angles: the environment alone came out 4–5 % dark in
 * linear light, every surface alike.
 */
export const ENV_UNITS = 1.05;

/**
 * The lights' highlights (the soft look's combine scales PlayCanvas's direct specular by it: the sun
 * and the lamps): PlayCanvas's GGX keeps the `1 / π` of the look's units while its light is the
 * look's / π (`SUN_UNITS`), so by the units alone it would be π. With the normal maps and the
 * Schlick term matched to the earlier pictures (`shaders/bump.ts`, `shaders/envBrdf.ts`), π still
 * leaves the noon shots too bright (orbit-day-0 +2.6 of 255 on average, 3.5–3.8 off) while it suits
 * the menus' golden hour, looking into the low sun (−3.9, against −6.1 at 2.4, which suits noon:
 * +1.0, 2.8–3.1 off). 2.8 is the best of both together (noon +1.9, golden hour −4.9; dusk and night
 * don't move). A string: it goes into the shaders.
 */
export const SPECULAR_UNITS = '2.8';
