/**
 * The look's daylight (`render/look.ts`, keyed in the Babylon renderer's units) in PlayCanvas's
 * units: one number per quantity, each with why it differs from Babylon's.
 *
 * Calibrated against the Babylon renderer's pictures (Stage 6 of the port: the bench poses with
 * `?diag=linear,nosims`, the lit colour before any image processing, and `?lightmix=sun,sky,env`
 * lighting one part at a time, at 6:36, 9, 13, 16, 18:30, 19:30, 20:12 and 23:00), after the parts
 * of the light that were not a matter of units were made Babylon's: the sky's harmonics as Babylon
 * evaluated them (`cargo envconv`), the haze (`shaders/fog.ts`), the AO map on the direct light
 * (`shaders/softLook.ts`) and the reflection's rise at a slant (`shaders/envBrdf.ts`).
 */

/**
 * The sun (and the moon): Babylon's PBR lights `albedo / π` per unit of a directional light's
 * intensity (Lambert: Oren-Nayar at no roughness), PlayCanvas `albedo`: the look's intensities / π.
 * (The lamps, `lamps.ts` `LAMP_UNITS`, are in the same units.)
 */
export const SUN_UNITS = 1 / Math.PI;

/**
 * The sky's hemispheric fill (`setSkyFill`, the soft look's `gameHemi`): Babylon's hemispheric
 * light has no `1 / π` in its PBR (`computeHemisphericDiffuseLighting` is the colour itself), and
 * neither has the game's term: the look's intensities as they are. (Its light alone at noon: within
 * 0.3 of 255 of Babylon's on average.)
 */
export const SKY_UNITS = 1;

/**
 * Image-based light (`scene.skyboxIntensity`: the sky's ambient harmonics and reflections): the
 * look's environment intensities, 5 % up. The harmonics are Babylon's irradiance exactly, but
 * PlayCanvas takes the specularity off the ambient once more (`dDiffuseLight * (1 − F0)`, on top of
 * the soft look's own `DIELECTRIC`) where Babylon took off its environment BRDF's reflectance, less
 * at most angles: the environment alone came out 4–5 % dark in linear light, every surface alike.
 */
export const ENV_UNITS = 1.05;

/**
 * The lights' highlights (the soft look's combine scales PlayCanvas's direct specular by it: the sun
 * and the lamps): PlayCanvas's GGX has Babylon's `1 / π` while its light is Babylon's / π
 * (`SUN_UNITS`), so by the units alone it would be π. Calibrated to 2.4: at π the noon sun's
 * highlight on the rough lawn near the camera came out brighter than Babylon's (+0.02 in linear;
 * PlayCanvas's normal maps are a little flatter, `bumpiness` mixing towards flat where Babylon
 * scales x and y, so the highlight is tighter), the morning's and afternoon's right; 2.4 splits it
 * (the sun alone at 9, 13 and 16: within 1.5 of 255 of Babylon's on average). A string: it goes
 * into the shaders.
 */
export const SPECULAR_UNITS = '2.4';
