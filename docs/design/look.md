# Look

*Design notes for how Idyll Lives looks. Inspired by classic life-simulation games, described
here on our own terms. Status: 0.3.1. This is the Medium graphics preset, everyone's default; Low
and High change a few things (softer or sharper shadows, fewer lamps, half the lawn, no shadows
from the woods, the street or the residents and the street's far models on Low, two mirrors
reflecting every frame on High), listed in
[graphics.md](graphics.md).*

The world has one look: soft, painterly shading with golden-hour light and blue moonlit nights.
All art is original or CC0.

Code: `web/src/render/look.ts` holds the look as data; `web/src/render/babylon/*` applies it, and
`softLook.ts` draws every lit surface in the soft shading below (Settings, Performance, Shading;
the `classic` option brings back the PBR shading it replaced).

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
- **Soft, painted surfaces.** Light falls off widely and gently, shade takes the sky's tint instead
  of going black, and a quiet rim keeps silhouettes readable: closer to a hand-painted illustration
  than a photograph, without turning flat or cel-shaded.

## Light and grading

| Aspect | Choice |
|---|---|
| Light | Saturated warm sun on an arc, lowered (`sunHeight` 0.8) for long, dramatic shadows. A low peach sunrise, a golden (not red) late afternoon, an orange sunset only as the sun touches the horizon, then a blue dusk with lamps coming on. |
| Nights | A blue moon fill with a strong night grade (0.85), so nights are clearly blue but not murky; warm lamps (`#FFC27A`) and glowing windows. |
| Grading | ACES tone mapping, contrast 1.1, saturation +14. Split toning: warm highlights (hue 38) and cool shadows (hue 222). |
| Effects | A light vignette and soft distance haze. |
| Shadows | Soft, filtered shadows from one sun shadow map over a fixed area round the lot. Its size comes with the graphics preset: 2048 on Medium (the default), 1024 on Low, 4096 on High. |

## Shading

Every lit PBR surface is drawn with the **soft look** (`softLook.ts`, a material plugin that works
on WebGPU and WebGL2). It keeps the light PBR computed — the sun and the lamps, with their cast
shadows — and redraws it as a wide, soft ramp in the light's own colour, so golden hour stays
golden, lamps stay warm and night stays dark. The environment (the sky, with its occlusion) fills
the shade at just over half, tinted cool, so shadows read blue-grey rather than black. A gentle
fresnel rim lifts silhouettes and a little extra saturation keeps the palette lively. There are no
ink outlines: the look is soft and illustrative, not cel-shaded.

The plugin replaces the material's colour just before fog and image processing, so ACES, the
exposure and the night grade still apply, and it runs after the wall cutaway and the basements.
Metals shade in their base colour (PBR gives them no diffuse colour); unlit materials keep their own
look. Water, the sky and glass panes that keep their reflections keep their own materials on
purpose.

It is the default (Settings, Performance, **Shading**). **Classic** draws the PBR shading it
replaced, and switching is live: the plugin takes its define from the setting whenever a material's
shaders are prepared, so the materials (the game's and the preview studios') are only marked dirty
to redraw them the other way. The item pictures in Furnish and the portraits are drawn in the chosen
shading and cached per shading; Furnish keeps its pictures between visits per app version, so a
change to the look's code bumps the picture store's version in `preview/items.ts`. On the dev
server `?look=soft|classic` overrides the setting.

## The world

| Element | Design |
|---|---|
| Houses | `house.ts` generates walls from the simulation layout: siding or brick per house, interior paint per room, a tiled wainscot in bathrooms, a stone plinth, corner boards, a frieze, baseboards, windows with muntins, shutters and sills, door casings, painted doors, and gable or hip roofs (34° pitch, 0.4 m overhang, closed gables). |
| Cutaway | Done on the GPU. Each wall vertex stores which view directions it hides, and the vertex shader lowers it to a 0.35 m stub. Roofs show only with walls up and when not zoomed in. |
| Night interiors | Every lamp gives warm light (clustered lighting, up to 32 lamps) on Medium and High; on Low only the four nearest the biggest rooms do, the storey in view first. A room with no lamp gets a dim fill. Lamps switch on and off (the player per lamp, residents per room as they come and go); a room whose lamps are all off goes dark at the windows. When everyone in a room is asleep its lights go out and its windows stop glowing, and a neighbour's windows glow only while one of its residents is home and up. Windows glow from outside at night only. |
| Sunlight indoors | The sun comes in through windows and doorways only: walls cast in every wall mode under a shadow-only ceiling. A room without windows is dark by day, as a basement is, and its lamps light it day and night. |
| Garden | A bright yellow-green lawn, foundation shrubs, lot trees, clouds and a sun glow in the sky. |
| Selection marker | A small marker over the selected resident. Its colour follows their mood through a five-step palette (`marker` in `look.ts`). |
| Residents | Made in code (`cargo characters`): smooth, slightly toy-like bodies (an adult's head about 1/6.5 of the height, hands and feet a little large) from a cage of rounded cross-sections, subdivided once, with a low-detail copy for far residents and shadows. Women and men are separate bodies; each life stage, from a baby to an elder, has its own proportions, and a build slider makes anyone slimmer or broader. The head never deforms: eyelids, brows, lips, teeth, tongue and the beard are small shapes moved by morphs (ARKit face units for moods, one whole-face recipe per emotion adapted from TalkingHead's expressions; idle micro-expressions; blinks and lids following the gaze; visemes for lip sync). Eyes are drawn by the shader (iris in the resident's eye colour, pupil, catchlight) and turn on bones. Eleven hairstyles, bald and a beard, built from caps, curtains, lobes and tails, in flat colour with soft clump bands; long hair and tails follow the neck and shoulders. Clothes are painted on the body by its shader (tops, bottoms and shoes in the creator's colours, with soft edges), with shells only where the silhouette needs them (jackets and suits, collars, skirts, soles and boot cuffs). Skin takes the creator's tone, warmed as real skin reflects, deep tones keeping enough value for their features to read; cavities deepen with age. Clips are made in code (`characters/clips`) with arm and leg IK: residents stand relaxed with the weight drifting from leg to leg, walk with planted feet and a hip sway (more in women), sit back onto seats, lie under a blanket, and carry dishes on their palms. See `tools/characters/README.md`. |

## Birds

Garden birds live about the viewed lot (`render/babylon/birds.ts`; presentation only, nothing in the simulation or the save): blue tits, chaffinches, willow warblers, blackbirds and collared doves, Blender-built (`tools/art/blender/families/birds.py`) in five rigid parts the renderer moves (body, head, tail and the two spread wings, hidden while perched). They are drawn 1.25–1.6× life size, the small ones most: at real size a tit is about 5 px from the game's usual camera, and the game's apples and flowers are a little larger than life too, so they still differ in size as they should.

- **Homes:** each bird belongs to a plant near the lot (the lot's garden trees and shrubs, garden things outdoors, the street's bushes and hedges, the woods near the town), as many as its size and the preset allow (`birds` in [graphics.md](graphics.md)), made from where it stands, so a rebuild brings back the same birds. Species go by habitat: tits, finches and warblers in broadleaved trees and bushes, blackbirds low in bushes and on the lawn, collared doves in pairs, high up and on the roof ridge (while the roof shows).
- **Perched:** on a perch point of the plant's model (manifest `perches`, on its outer twigs), swaying as the wind bends the branch (the vertex shader's bend, worked out on the CPU). Small jerky glances of the head, a cock of it, a flick of the tail (a blackbird holds it up a moment), preening now and then, a hop to a nearby twig or a turn on the spot. Small birds fidget; doves sit still and long. A bird that sings lifts its head and holds still.
- **Flying:** hops within a plant, flights to another plant, the lawn or the roof, along a curve lifted over any house in the way (walls and roof, and never inside; a flight that would have to climb a wall is not taken). A burst to take off; tits, finches and warblers bound (beating on the rise, wings shut on the dip), blackbirds fly low and direct, doves flap steadily with glides; they bank into turns and flare to land.
- **On the lawn:** blackbirds run and stop to listen, chaffinches hop, doves walk with their heads bobbing; they peck and look up. Never on paths, patios, in rooms or under furniture.
- **Wary:** a resident within a few metres (3–4 m on the lawn, less up in a bush), or the camera within 1.8 m, sends them up into the nearest plant away from it, high; they keep off the lawn for 20–60 s.
- **The day:** busiest at dawn and early evening, quieter at midday. At dusk each flies home and roosts deep in its plant, head turned back into its feathers, puffed up; they wake at first light (blackbirds first, doves last), when the dawn chorus starts.
- **Time:** their clock runs with the game's: paused, they hold still (perched ones still sway with their branches); at speeds 3–5 they hurry only a little (1.5–1.75×). **Reduce motion:** no flights or hops, only the way home from the lawn, slowly; slow, eased head turns and no tail flicks.
- **Song:** the birds in view sing from where they sit (`audio/world.ts`): placed left to right as on screen, louder nearer; a few unseen ones further off keep a softer chorus behind them. Each species has its own song (`audio/birdsong.ts`), the blackbird's a fluting, varied phrase that trails off into a twitter.
- The menus' town has none (at that distance a bird would be under a pixel).
