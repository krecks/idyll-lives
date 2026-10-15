# Look

*Design notes for how Idyll Lives looks. Inspired by classic life-simulation games, described
here on our own terms. Status: 0.2.0. This is the Medium graphics preset, everyone's default; Low
and High change a few things (softer or sharper shadows, fewer lamps, half the lawn and no woods'
shadows on Low, two mirrors reflecting every frame on High), listed in
[graphics.md](graphics.md).*

The world has one look: warm, painterly realism with golden-hour light and blue moonlit nights.
All art is original or CC0.

Code: `web/src/render/look.ts` holds the look as data; `web/src/render/babylon/*` applies it.

## Goals

- **A warm, lived-in neighbourhood.** Houses, gardens and people should read as a small,
  cared-for world: a warm, stylised-realistic look rather than photorealism or flat cartoon.
- **Time of day you can feel.** The look is driven by time-of-day keyframes (sun, sky fill,
  image-based light, horizon, zenith, exposure and lamps every few hours), so mornings, golden
  hour, blue hour and night each have their own mood.
- **Only the sun turns warm at sunset.** The sky fill and ambient light stay neutral to cool, and
  the horizon haze stays pale and desaturated. Shadows read blue, greens stay green, and only
  sunlit faces turn golden. The sun fades out around sunrise and sunset, so the switch between
  sun and moon never pops.
- **Warm lamps against cool nights.** At night a moonlight grade turns dark tones blue-grey
  (grass alone would otherwise read dark green) while lamps and lit windows stay warm.
- **Readable at a glance.** The dollhouse view has to show who is where and what they are doing,
  so clarity beats extra effects. There is no bloom or tilt-shift blur.

## Light and grading

| Aspect | Choice |
|---|---|
| Light | Saturated warm sun on an arc, lowered (`sunHeight` 0.8) for long, dramatic shadows. A low peach sunrise, a golden (not red) late afternoon, an orange sunset only as the sun touches the horizon, then a blue dusk with lamps coming on. |
| Nights | A blue moon fill with a strong night grade (0.85), so nights are clearly blue but not murky; warm lamps (`#FFC27A`) and glowing windows. |
| Grading | ACES tone mapping, contrast 1.1, saturation +14. Split toning: warm highlights (hue 38) and cool shadows (hue 222). |
| Effects | A light vignette and soft distance haze. |
| Shadows | Soft, filtered shadows from one sun shadow map over a fixed area round the lot. Its size comes with the graphics preset: 2048 on Medium (the default), 1024 on Low, 4096 on High. |

## The world

| Element | Design |
|---|---|
| Houses | `house.ts` generates walls from the simulation layout: siding or brick per house, interior paint per room, a tiled wainscot in bathrooms, a stone plinth, corner boards, a frieze, baseboards, windows with muntins, shutters and sills, door casings, painted doors, and gable or hip roofs (34° pitch, 0.4 m overhang, closed gables). |
| Cutaway | Done on the GPU. Each wall vertex stores which view directions it hides, and the vertex shader lowers it to a 0.35 m stub. Roofs show only with walls up and when not zoomed in. |
| Night interiors | Every lamp gives warm light (clustered lighting, up to 32 lamps) on Medium and High; on Low only the four nearest the biggest rooms do, the storey in view first. A room with no lamp gets a dim fill. When everyone in a room is asleep its lights go out and its windows stop glowing. Windows glow from outside at night only. |
| Garden | A bright yellow-green lawn, foundation shrubs, lot trees, clouds and a sun glow in the sky. |
| Selection marker | A small marker over the selected resident. Its colour follows their mood through a five-step palette (`marker` in `look.ts`). |
| Residents | Made in code (`tools/characters/prim`): smooth, slightly toy-like bodies (an adult's head about 1/6.5 of the height, hands and feet a little large) from a cage of rounded cross-sections, subdivided once, with a low-detail copy for far residents and shadows. Women and men are separate bodies; each life stage, from a baby to an elder, has its own proportions, and a build slider makes anyone slimmer or broader. The head never deforms: eyelids, brows, lips, teeth, tongue and the beard are small shapes moved by morphs (ARKit face units for moods, one whole-face recipe per emotion adapted from TalkingHead's expressions; idle micro-expressions; blinks and lids following the gaze; visemes for lip sync). Eyes are drawn by the shader (iris in the resident's eye colour, pupil, catchlight) and turn on bones. Eleven hairstyles, bald and a beard, built from caps, curtains, lobes and tails, in flat colour with soft clump bands; long hair and tails follow the neck and shoulders. Clothes are painted on the body by its shader (tops, bottoms and shoes in the creator's colours, with soft edges), with shells only where the silhouette needs them (jackets and suits, collars, skirts, soles and boot cuffs). Skin takes the creator's tone, warmed as real skin reflects, deep tones keeping enough value for their features to read; cavities deepen with age. Clips are made in code (`characters/clips`) with arm and leg IK: residents stand relaxed with the weight drifting from leg to leg, walk with planted feet and a hip sway (more in women), sit back onto seats, lie under a blanket, and carry dishes on their palms. See `tools/characters/README.md`. |
