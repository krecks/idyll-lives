# Third-party assets

All of the third-party *art* below is **CC0 1.0 (public domain)**. You don't have to credit it, but we list it so it's easy to trace and replace. The voice model, the dictionary, the language model, the typeface and the decoders further down have their own licences, and so do the libraries the game is built with (`../THIRD-PARTY-LICENSES.txt`, written by every build: each one with its licence text).
Downloaded 2026-10-07. Textures were resized and re-encoded (JPG). Fabric, wood, foliage, grass, wall, siding, roof, carpet, wallpaper and painted-wood albedo maps were converted to greyscale "detail" maps, so manifest, house and room colours set the hue.

## Textures (`textures/`)

| Folder | Source asset | Authors | URL | Licence |
|---|---|---|---|---|
| `floor_laminate/` | Laminate Floor 02 | Dario Barresi, Charlotte Baglioni | https://polyhaven.com/a/laminate_floor_02 | CC0 |
| `wall_plaster/` | Painted Plaster Wall | Amal Kumar | https://polyhaven.com/a/painted_plaster_wall | CC0 |
| `road_asphalt/` | Asphalt 02 | Rob Tuytel | https://polyhaven.com/a/asphalt_02 | CC0 |
| `path_hexpaving/` | Hexagonal Concrete Paving | Stephan Seeliger | https://polyhaven.com/a/hexagonal_concrete_paving | CC0 |
| `fabric_linen/` | Rough Linen | colormass, Rico Cilliers | https://polyhaven.com/a/rough_linen | CC0 |
| `wood_oak/` | Oak Veneer 01 | Jenelle van Heerden | https://polyhaven.com/a/oak_veneer_01 | CC0 |
| `bark/` | Bark Brown 02 | Rob Tuytel | https://polyhaven.com/a/bark_brown_02 | CC0 |
| `foliage/` | Forest Leaves 03 | Rob Tuytel, Dimitrios Savva | https://polyhaven.com/a/forest_leaves_03 | CC0 |
| `grass/` | Grass 001 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Grass001 | CC0 |
| `siding_wood/` | Wood Siding 009 (greyscale detail, board rows staggered) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=WoodSiding009 | CC0 |
| `brick/` | Red Brick 03 | Rob Tuytel | https://polyhaven.com/a/red_brick_03 | CC0 |
| `roof_shingles/` | Grey Roof Tiles 02 (greyscale detail) | Rob Tuytel | https://polyhaven.com/a/grey_roof_tiles_02 | CC0 |
| `stone_foundation/` | Rustic Stone Wall 02 | Amal Kumar | https://polyhaven.com/a/rustic_stone_wall_02 | CC0 |
| `floor_wood/` | Wood Floor 040 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=WoodFloor040 | CC0 |
| `floor_tile/` | Tiles 141 | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Tiles141 | CC0 |
| `carpet/` | Carpet 012 (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Carpet012 | CC0 |
| `wallpaper/` | Plaster 001 (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Plaster001 | CC0 |
| `painted_wood/` | Painted Wood 008 A (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=PaintedWood008A | CC0 |
| `grass_lush/` | Grass 004 (greyscale detail) | ambientCG (Lennart Demes) | https://ambientcg.com/view?id=Grass004 | CC0 |

## Environment (`env/`)

| File | Source asset | Authors | URL | Licence |
|---|---|---|---|---|
| `belfast_open_field.pcenv` | Belfast Open Field (1k HDRI), converted by `cargo envconv` | Dimitrios Savva, Jarod Guest | https://polyhaven.com/a/belfast_open_field | CC0 |

## Models

Every model of a thing in the world (furniture, appliances, decor, the town's street furniture and
shops, trees, bushes, flowers and garden things, food and kitchen tools) is built in Blender by the
family scripts under `tools/art/blender/families/` on the `gamekit` toolkit and exported as `.glb`
(glTF, original, CC0). They take their surfaces from the finishes and textures above. No downloaded
models are used.

The garden decor (`models/garden/`, override pack `packs/gardenDecor/manifest.json`) is built in Blender by
`tools/art/blender/families/garden_decor.py` on the shared `tools/art/blender/gamekit/` toolkit and exported as `.glb`
(glTF, original, CC0).

The houseplants (`models/plants/plant.glb`, `monstera.glb`, `fiddleFig.glb`, `snakePlant.glb`, `cacti.glb`,
`succulents.glb`, `bostonFern.glb`, `palm.glb`, `orchid.glb`, `bonsai.glb`, override pack
`packs/plants/manifest.json`) are built in Blender by `tools/art/blender/families/plants.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). Their leaves sway in the nature shader's wind; the bonsai's stray sprig
shrinks away while a resident prunes it.

The flower beds (`models/plants/flowerbed.glb`, `tulips.glb`, `daffodils.glb`, `daisies.glb`, `sunflowers.glb`,
`poppies.glb`, `marigolds.glb`, `pansies.glb`, `lilies.glb`, `lupins.glb`, `forgetMeNots.glb`, `cosmos.glb`,
`meadow.glb`, `geraniums.glb`, `flowerBox.glb`, in the same pack) are built by the same script, also original and
CC0. A flower in the wildflower meadow shrinks away while a resident picks it.

The shrubs (`models/plants/bush.glb`, `boxwood.glb`, `topiaryCone.glb`, `hedge.glb`, `hydrangea.glb`, `rosebush.glb`,
`azalea.glb`, `lavender.glb`, `fern.glb`, `grass.glb`, `hosta.glb`, the lot dressing's `bushSmall.glb` and
`bushFlowering.glb` and the streets' `streetHedge.glb`, in the same pack) are built by the same script,
also original and CC0, each with a far version (`<id>.far.glb`), as are the flower beds. A trim takes the clipped
shrubs' stray sprigs off, a prune cuts a rose and a cut a few lavender stems.

The edibles (`models/plants/vegPatch.glb`, `tomatoes.glb`, `strawberries.glb`, `herbs.glb`, `pumpkins.glb`, each
with its far version, in the same pack) are built by the same script, also original and CC0. The four crops grow
from seedlings to ripe and lose their crop when harvested (`grow`); the herb planter's basil shrinks while herbs are
picked; a harvested vegetable patch is bare.

The trees (`models/trees/oak.glb`, `maple.glb`, `birch.glb`, `cherry.glb`, `apple.glb`, `japaneseMaple.glb`,
`magnolia.glb`, `willow.glb`, `pine.glb`, `spruce.glb`, `cypress.glb`, `lemonTree.glb`, each with its far version,
override pack `packs/trees/manifest.json`) are built in Blender by `tools/art/blender/families/trees.py` on `gamekit`
and exported as `.glb` (glTF, original, CC0). They stand on the lots and in the woods, whose far trees draw the far
versions. An apple or a lemon shrinks away while it is picked, the Japanese maple's stray sprigs while it is shaped. The pine, spruce, Japanese maple, willow, cypress, maple and magnolia were rebuilt for the authenticity review
(G-38, 2026-10-21): crowns over head height, the willow's streamers, a slim cypress, a maple of its own and the
magnolia's goblets; with them the bonsai's stand, the rose bush, azalea, meadow and strawberries (`plants.py`) and
the garden boulders (`garden_decor.py`), all original and CC0.

The garden birds (`models/birds/blueTit.glb`, `chaffinch.glb`, `warbler.glb`, `blackbird.glb`, `collaredDove.glb`,
override pack `packs/birds/manifest.json`) are built in Blender by `tools/art/blender/families/birds.py` on `gamekit`
and exported as `.glb` (glTF, original, CC0): a blue tit, a male chaffinch, a willow warbler, a male blackbird and a
collared dove at their field-guide sizes, each in five parts the game moves (body, head, tail, the two wings). The
places they perch on the trees, shrubs and hedges (`perches` in the trees' and plants' packs) are found on those
models by the same scripts (`gamekit/perch.py`), data only.

The kitchen fridge (`models/kitchen/fridge.glb`, override pack `packs/kitchen/manifest.json`) is built in
Blender by `tools/art/blender/families/kitchen.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).

The fold-down wall bed (`models/living/wallBed.glb`, pack `packs/living/manifest.json`) is built in Blender by
`tools/art/blender/families/living.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).

The sofa bed (`models/living/sofaBed.glb`, same pack) is built in Blender by `tools/art/blender/families/living.py`
on `gamekit` and exported as `.glb` (glTF, original, CC0), with a `bed_out` clip (the mattress pulling out) held
while it's open.

The sofas and sectionals (`models/living/sofa.glb`, `loveseat.glb`, `sofaThree.glb`, `sofaDesign.glb`,
`sofaThreeSeat.glb`, `sectionalMiddle.glb`, `sectionalArmLeft.glb`, `sectionalArmRight.glb`, `sectionalCorner.glb`,
`chaise.glb`, same pack) are built in Blender by `tools/art/blender/families/living.py` on `gamekit` and exported as
`.glb` (glTF, original, CC0). Their seat cushions sink in while they're sat or dozed on (`sink_in`).

The armchairs and small seats (`models/living/armchair.glb`, `armchairLounge.glb`, `armchairLeather.glb`,
`armchairDesign.glb`, `armchairWingback.glb`, `recliner.glb`, `rockingChair.glb`, `pouf.glb`, `ottoman.glb`,
`floorCushion.glb`, `beanBag.glb`, same pack) are built the same way (glTF, original, CC0). Their seats sink in, the
recliner tips back onto its head cushion with its footrest up, the rocking chair rocks with whoever sits in it, and the bean bag squashes;
the designer lounge chair is still.

The coffee and side tables (`models/living/coffeeTable.glb`, `coffeeTableGlass.glb`, `coffeeTableMarble.glb`,
`sideTable.glb`, `coffeeTableRound.glb`, `coffeeTableLong.glb`, `nestingTables.glb`, `sideTableRound.glb`,
`consoleTable.glb`, same pack) are built the same way (glTF, original, CC0). The coffee table's drawer slides out
with a magazine in it; the rest are still.

The shelves (`models/living/bookshelf.glb`, `bookcaseLow.glb`, `bookcaseWide.glb`, `shelvingIndustrial.glb`,
`bookcaseTall.glb`, `bookcaseCorner.glb`, `cubeShelf.glb`, `ladderShelf.glb`, `displayCabinet.glb`, same pack) are
built the same way (glTF, original, CC0). On the bookcases one book tips out while a resident reads there; the
rest are still.

The staircase (`models/living/stairs.glb`, same pack) is built the same way (glTF, original, CC0): still, its steps
where the house's are.

The bathroom sinks (`models/bathroom/sink.glb`, `sinkPedestal.glb`, `vanityDouble.glb`, `sinkVessel.glb`,
`sinkConsole.glb`, override pack `packs/bathroom/manifest.json`) are built in Blender by
`tools/art/blender/families/bathroom.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each has a
`water_on` clip (the tap stream) played while a resident washes. The double vanity keeps its mirror glass as a
primitive `part` (a file can't reflect yet); its cabinet and mirror frames are in the file.

The bathroom toilets (`models/bathroom/toilet.glb`, `toiletCompact.glb`, `toiletSmart.glb`, `toiletClassic.glb`,
`toiletWallHung.glb`, override pack `packs/bathroom/manifest.json`) are built in Blender by
`tools/art/blender/families/bathroom.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each lifts
its lid (`lid_up`) while a resident uses it.

The bathroom showers (`models/bathroom/shower.glb`, `showerStall.glb`, `showerWalkIn.glb`, `showerRain.glb`, override
pack `packs/bathroom/manifest.json`) are built in Blender by `tools/art/blender/families/bathroom.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). Each rains from its head (`water_on`) while a resident showers; the
shower's glass door slides open to let them in and out, and the stall's curtain is drawn across while they shower.

The bathroom baths (`models/bathroom/bathtub.glb`, `bathtubClawfoot.glb`, `bathtubFreestanding.glb`, override pack
`packs/bathroom/manifest.json`) are built in Blender by `tools/art/blender/families/bathroom.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). Each runs its tap and fills with water (`tap_on`, `fill`) while a resident
bathes, and drains after.

The bookworm pack models (`models/pack_bookworm/readingNook.glb`, `libraryWall.glb`, `writingBureau.glb`,
`teaTrolley.glb`, `firesideLounge.glb`, `bedsideStack.glb`, override pack `packs/pack_bookworm/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_bookworm.py` on `gamekit` and exported as `.glb` (glTF,
original, CC0). Each moves one part while it is used: the nook's cushion sinks, the ladder rolls, the bureau's flap
drops, the teapot pours, the fire grows and the top book opens.

The outgoing pack models (`models/pack_outgoing/partyBar.glb`, `conversationSectional.glb`, `karaokeStage.glb`,
`videoCallNook.glb`, `danceFloor.glb`, `photoWall.glb`, override pack `packs/pack_outgoing/manifest.json`) are built in
Blender by `tools/art/blender/families/pack_outgoing.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
Each moves while it is used: the bar's tap lever pulls, the sectional's cushions sink, the mics rise, the ring light
swings up, the dance floor's tiles pulse and a new photo grows.

The romantic pack models (`models/pack_romantic/candlelitTable.glb`, `heartTub.glb`, `vanity.glb`, `gramophone.glb`,
`letterBureau.glb`, `roseArbourSwing.glb`, override pack `packs/pack_romantic/manifest.json`) are built in Blender by
`tools/art/blender/families/pack_romantic.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each moves
while it is used: the candles light, the tub fills with petals, the vanity's drawer opens, the gramophone's arm drops
onto the turning record, the bureau's flap drops and the arbour swing sways while a bloom is cut. The candlelit table
was rebuilt with a chair at each end, its two diners facing each other (2026-10-21).

The slob pack models (`models/pack_slob/microwaveCart.glb`, `beanbag.glb`, `snackNightstand.glb`, `spritzStation.glb`,
`arcadeCabinet.glb`, `gamingRecliner.glb`, override pack `packs/pack_slob/manifest.json`) are built in Blender by
`tools/art/blender/families/pack_slob.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each moves while
it is used: the microwave's door opens, the beanbag squashes, the snack drawer slides out, the spray tips, the joystick
waggles, and the recliner reclines and swings its cup holder out.

The example pack models (`models/pack_example/chefStove.glb`, `whirlpoolTub.glb`, override pack
`packs/pack_example/manifest.json`) are built in Blender by `tools/art/blender/families/pack_example.py` on `gamekit`
and exported as `.glb` (glTF, original, CC0). The stove's oven door drops and its front burner lights while a resident
cooks; the tub fills and bubbles while residents soak.

The foodie pack models (`models/pack_foodie/herbGarden.glb`, `deliPantry.glb`, `espressoBar.glb`,
`cookbookLectern.glb`, `pizzaOven.glb`, `chefsIsland.glb`, override pack `packs/pack_foodie/manifest.json`) are built
in Blender by `tools/art/blender/families/pack_foodie.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
A sprig grows, the pantry doors swing, the coffee runs, a page turns, the oven door comes off for the peel as the
fire grows, and the island's drawer pulls out.

The gloomy pack models (`models/pack_gloomy/recordCabinet.glb`, `cello.glb`, `journalDesk.glb`, `rainLamp.glb`,
`windowSeat.glb`, `soakingTub.glb`, override pack `packs/pack_gloomy/manifest.json`) are built in Blender by
`tools/art/blender/families/pack_gloomy.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The cabinet's
lid lifts and its record turns, the cello comes off its stand, a candle is lit, the rain runs down the lamp over a
rippling pool, the window seat's cushion sinks and the tub fills.

The loner pack models (`models/pack_loner/quietPod.glb`, `shortwaveRadio.glb`, `puzzleTable.glb`,
`bonsaiBench.glb`, `soloSauna.glb`, `deliveryHatch.glb`, override pack `packs/pack_loner/manifest.json`) are built
in Blender by `tools/art/blender/families/pack_loner.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
Each moves a part while it is used: the pod's hood swings over, the radio's dial turns and its antenna rises, a
patch of the puzzle is laid, a twig of the bonsai is trimmed, the sauna's door opens and the delivery hatch opens.

The nature lover pack models (`models/pack_natureLover/veggieBed.glb`, `pottingBench.glb`, `birdFeeder.glb`,
`gardenHammock.glb`, `cedarShower.glb`, `mossTerrarium.glb`, `glasshouse.glb`, override pack
`packs/pack_natureLover/manifest.json`) are built in Blender by `tools/art/blender/families/pack_natureLover.py` on
`gamekit` and exported as `.glb` (glTF, original, CC0). Each moves while it is used: a lettuce is picked from the
veggie bed (whose crop follows its growth stage), a seedling comes up, a bird lands, the hammock sways, the shower
runs, the cloche lifts and the glasshouse's door and roof vent open.

Rebuilt in the same families (2026-10-21, original, CC0): the entry console, the baking counter and the spritz
station with back panels for their mirrors, hooks and shelves; the organizer desk and the love-letter bureau turned
round, their backs on the wall; the two pack showers with glass doors that open; the chef's island with counter
stools and a hob that heats; the two hammocks on 3 × 1 stands; the nap pod, the quilt table, the vibration plate and
the sauna with room for the body; a real zafu; the writing desks at desk height; the fireside hood over its fire; a
nightstand-sized bedside stack; the lava lamp turned to its watcher; and glowing screens, lamps, tiles and candles.

The neat pack models (`models/pack_neat/steamMopDock.glb`, `laundryCenter.glb`, `rainShowerTower.glb`,
`organizerDesk.glb`, `robotVacuumDock.glb`, `entryConsole.glb`, override pack `packs/pack_neat/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_neat.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). Each moves one part while it is used: the mop leaves its dock, the washer's door opens, the rain falls, a bin
slides out of the desk's hutch, the robot rolls out and back, and the key tray drawer opens.

The cheerful pack models (`models/pack_cheerful/jukebox.glb`, `karaokeMachine.glb`, `mocktailCart.glb`,
`rainbowHammock.glb`, `sunriseLamp.glb`, `partyBooth.glb`, override pack `packs/pack_cheerful/manifest.json`) are built
in Blender by `tools/art/blender/families/pack_cheerful.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). Each moves one part while it is used: the record swings up, the microphone rises, the shaker tips, the hammock
sways with its sleeper, the sun rises and the booth's curtain is drawn.

The couch potato pack models (`models/pack_couchPotato/cinemaWall.glb`, `gameConsole.glb`, `recliner.glb`,
`snackFridge.glb`, `fitnessGame.glb`, `lavaLamp.glb`, override pack `packs/pack_couchPotato/manifest.json`) are built
in Blender by `tools/art/blender/families/pack_couchPotato.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). The cinema's curtains are drawn apart, the console's disc tray slides out, the recliner reclines, the fridge
door swings, the balance board tilts and the lava lamp's wax rises while it is used, and drifts round all the time.

The energetic pack models (`models/pack_energetic/boulderingWall.glb`, `spinBike.glb`, `trampoline.glb`,
`streetballHoop.glb`, `powerBlender.glb`, `contrastShower.glb`, `recoveryPod.glb`, override pack
`packs/pack_energetic/manifest.json`) are built in Blender by `tools/art/blender/families/pack_energetic.py` on
`gamekit` and exported as `.glb` (glTF, original, CC0). The bike's cranks turn and the trampoline's mat dips while
they are used, the hoop's net sways, the blender's lid lifts, the shower's rain grows and the recovery pod's canopy
lowers; the bouldering wall is still.

The hot-headed pack models (`models/pack_hotHeaded/heavyBag.glb`, `smashBin.glb`, `thunderDrums.glb`,
`coolDownCushion.glb`, `grudgeJournal.glb`, `coldPlunge.glb`, override pack `packs/pack_hotHeaded/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_hotHeaded.py` on `gamekit` and exported as `.glb` (glTF,
original, CC0). Each moves while it is used: the bag swings, the crate's lid flips up, the cymbals rock, the zafu
sinks, the desk drawer slides out and the plunge cover rolls back.

The kind pack models (`models/pack_kind/neighbourlyOven.glb`, `givingJar.glb`, `knittingRocker.glb`,
`welcomeTeaTable.glb`, `songbirdFeeder.glb`, `hearthSettee.glb`, override pack `packs/pack_kind/manifest.json`) are
built in Blender by `tools/art/blender/families/pack_kind.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). Each moves while it is used: the oven door drops, a coin drops into the jar, the rocker rocks, the teapot pours,
the feeder's roof lifts and songbirds fly out, and the settee's cushions sink. The tea table was rebuilt lower and
narrower, its poufs either side of it facing each other (2026-10-21).

The lazy pack models (`models/pack_lazy/snoozecloudBed.glb`, `napPod.glb`, `quiltTable.glb`, `deliveryLocker.glb`,
`vibePlate.glb`, `smartHub.glb`, override pack `packs/pack_lazy/manifest.json`) are built in Blender by
`tools/art/blender/families/pack_lazy.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each moves while
it is used: the bed's head section rises, the pod's hood swings over, the quilt's edge lifts, a locker door opens, the
plate's deck shakes and the tablet tilts up.

The laundry models (`models/laundry/laundryWasher.glb`, `laundryDryer.glb`, `towelRail.glb`, `towelLadder.glb`,
`laundryBasket.glb`, `bathMat.glb`, and the things carried doing the laundry, `washingBasket.glb` and
`washingPiece.glb`, override pack `packs/laundry/manifest.json`) are built in Blender by
`tools/art/blender/families/laundry.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The washer and
the dryer open their door while a resident loads or empties them and turn their drum while they run; the rail,
ladder, basket and mat are still.

The hobby models (`models/hobbies/piano.glb`, `ukulele.glb`, `guitarElectric.glb`, `keyboardDigital.glb`,
`drumKit.glb`, `guitarAcoustic.glb`, `chessTable.glb`, `poolTable.glb`, `foosball.glb`, `dartboard.glb`,
`arcadeCabinet.glb`, `puzzleTable.glb`, `easel.glb`, `workbench.glb`, `telescope.glb`, `sewingTable.glb`,
`chemistrySet.glb`, `potteryWheel.glb`, `craftTable.glb`, override pack `packs/hobbies/manifest.json`) are built in
Blender by `tools/art/blender/families/hobbies.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The
piano lifts its fallboard, the drums' cymbals, the foosball rods and the arcade joystick move while played, darts,
a painting, a jigsaw patch, a pot and a flask's liquid grow, and the chess pawn, the pool rack, the vice, the
telescope, the sewing machine's presser and handwheel, the pottery wheel and the scrapbook's cover move while a
resident uses them; the guitars, the ukulele and the digital piano are still.

The kids' things (`models/kids/*.glb`: the crib, `kidsBed`, `kidsBedCar`, `highChair`, the toys `toyBox`, `toyChest`,
`dollHouse`, `rockingHorse`, `playTent`, `ballPit`, `trainTable`, `playMat`, `teddyBear`, `beanBagKids`, and
`kidsDesk`, `kidsTable`, `kidsChair`, `kidsBookshelf`, `kidsChalkboard`, `kidsEasel`, override pack
`packs/kids/manifest.json`) are built in Blender by `tools/art/blender/families/kids.py` on `gamekit` and exported as
`.glb` (glTF, original, CC0). The crib lowers its front rail for feeding, changing and cuddles and turns its mobile
over a sleeping baby; the beds turn their duvet down; the boxes, the chest and the doll house open; the horse rocks
its rider, the train goes round its track, the ball pit's balls bob; the tent's flap rolls up, a tower of blocks,
a chalk drawing and a painting appear, a picture book tips out, and the teddy and the bean bag squash.

The outdoor patio models (`models/outdoor/bench.glb`, `patioSet.glb`, `picnicTable.glb`, `gardenChair.glb`,
`patioTableRound.glb`, `patioTable4.glb`, `patioChair.glb`, `adirondackChair.glb`, `benchGarden3.glb`,
`outdoorBar.glb`, `parasol.glb`, override pack `packs/outdoor/manifest.json`) are built in Blender by
`tools/art/blender/families/outdoor.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The parasol's
canopy stirs in the wind (`stir`); the rest are still.

The outdoor loungers and swings (`models/outdoor/sunLounger.glb`, `hammock.glb`, `porchSwing.glb`, `swingSet.glb`,
`hotTubCedar.glb`, same pack and family) are built the same way (glTF, original, CC0). The lounger's back reclines
for sunbathing, the hammock and the swings sway with whoever is on them, and the hot tub's cover folds back for a
soak and hangs behind it while the water bubbles. The patio things, the hammock, the swings, the hot tub and the
grill were rebuilt to real sizes (2026-10-21).

The play and yard things (`models/yard/sandbox.glb`, `bikeRack.glb`, `raisedBed.glb`, `shedSmall.glb`,
`greenhouse.glb`, `compostBin.glb`, `rainBarrel.glb`, `clothesline.glb`, `firewoodStack.glb`, each with its
`.far.glb`, override pack `packs/yard/manifest.json`) are built in Blender by `tools/art/blender/families/yard.py`
on `gamekit` and exported as `.glb` (glTF, original, CC0). A sand castle grows in the sandpit, the bike lifts and
turns its back wheel, the raised bed's crop grows and is picked, the shed's and the greenhouse's doors open, the
compost bin's lid lifts and the washing goes out on the line.

The grills and the fire pit (`models/outdoor/grill.glb`, `grillStation.glb`, `firePit.glb`, same pack and family)
are built the same way (glTF, original, CC0). The grill's lid and the station's hood swing up while residents cook,
and the fire pit's flames grow on its logs while they warm up by it.

The home gym (`models/fitness/treadmill.glb`, `weightBench.glb`, `exerciseBike.glb`, `yogaMat.glb`,
`dumbbells.glb`, override pack `packs/fitness/manifest.json`) is built in Blender by
`tools/art/blender/families/fitness.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The treadmill
runs its belt and the bike turns its pedals while used, and the yoga mat unrolls for yoga; the bench and the
dumbbell rack are still.

The lamps (`models/lighting/*.glb`: the seven ceiling lights, three wall lights, five floor lamps and six table
lamps, override pack `packs/lighting/manifest.json`) are built in Blender by `tools/art/blender/families/lighting.py`
on `gamekit` and exported as `.glb` (glTF, original, CC0). Their shades glow at night (coloured glass in its own colour); the ceiling fan's blades
turn while it is on.

The decor models (`models/decor/*.glb`: the D1 wall art and clocks, D2 ornaments and D3 curtains and rugs, override
pack `packs/decor/manifest.json`) are built in Blender by `tools/art/blender/families/decor.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). The wall clock's second hand and the grandfather clock's pendulum go
round near the camera, the hanging plant sways when watered, the fire burns while a resident warms up by it, and the
curtains (a window-length pair, a wide pair and a floor-length pair) are drawn at night.

The bathroom mirror cabinet, round mirror and tall cabinet (`models/bathroom/mirrorCabinet.glb`, `mirrorRound.glb`,
`bathCabinetTall.glb`, override pack `packs/bathroom/manifest.json`) are built in Blender by
`tools/art/blender/families/bathroom.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The mirrors'
glass is in the files and reflects the room live on High.

The kitchen farmhouse and double counter sinks (`models/kitchen/farmhouseSink.glb`, `counterSinkDouble.glb`, pack
`packs/kitchen/manifest.json`) are built in Blender by `tools/art/blender/families/kitchen.py` alongside the
fridge, on `gamekit`, and exported as `.glb` (glTF, original, CC0), also with a `water_on` clip.

The other kitchen fridges (`models/kitchen/fridgeMini.glb`, `fridgeBuiltIn.glb`, `fridgeFrenchDoor.glb`,
`fridgeRetro.glb`, `fridgeMiniRetro.glb`, `wineFridge.glb`, same pack) are built in Blender by
`tools/art/blender/families/kitchen.py`
on `gamekit` and exported as `.glb` (glTF, original, CC0). Each hinges a door built closed and swings it open
while a resident takes food or a drink, held for a few seconds; the French-door freezer drawer slides, and every
freezer opens for ice cream. The wine fridge's glass door shows its bottles, and its top shelf of them slides out
once the door is open; the French-door fridge's doors sit flush on its cabinet (2026-10-21).

The kitchen cookers (`models/kitchen_cooking/stoveElectric.glb`, `stoveGas.glb`, `wallOven.glb`,
`rangeCooker.glb`, `cooktopCounter.glb`, `microwave.glb`, override pack `packs/kitchen_cooking/manifest.json`)
are built in Blender by `tools/art/blender/families/kitchen_cooking.py` on `gamekit` and exported as `.glb`
(glTF, original, CC0). Each drops an oven door on its bottom hinge while a resident bakes and turns a knob
while they cook, with a ring of blue flame growing on the gas ones; the oven tower's tray slides out, and the
microwave swings its door and turns its turntable.

The kitchen small appliances and dishwashers (`models/kitchen_cooking/coffeeMachine.glb`,
`breakfastCounter.glb`, `dishwasher.glb`, `dishwasherCompact.glb`, same pack) are built in Blender by
`tools/art/blender/families/kitchen_cooking.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
The coffee machine slides its cup under the spout and runs a coffee stream; the breakfast counter's toaster
lever slides and its toast pops, and its blender lid lifts over a swirling smoothie; the dishwashers drop their door and slide a rack
out. The dishwashers were rebuilt 60 and 45 cm wide beside a cupboard or a drawer stack (2026-10-21).

The kitchen base counters (`models/kitchen_counters/counter.glb`, `counterCorner.glb`, `counterEnd.glb`,
`counterCornerOuter.glb`, `counterDrawers.glb`, `counterOpenShelf.glb`, `butcherBlock.glb`,
`counterMarble.glb`, override pack `packs/kitchen_counters/manifest.json`) are built in Blender by
`tools/art/blender/families/kitchen_counters.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). The plain, corner, marble and butcher-block counters slide their top (or knife) drawer out 25 cm
while a resident makes a sandwich; the others are static base units with doors, drawers or open shelves.

The kitchen tall and wall storage (`models/kitchen_counters/pantry.glb`, `pantryTall.glb`, `wallCabinet.glb`,
`wallCabinetGlass.glb`, `wallCabinetCorner.glb`, `rangeHood.glb`, `spiceRack.glb`, `potRail.glb`,
`kitchenShelfWall.glb`, `trashCan.glb`, `recyclingBins.glb`, same pack) are built in Blender by
`tools/art/blender/families/kitchen_counters.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
The pantry and the tall larder swing both doors open 100° while a resident fetches a snack; the wall
cabinets, cooker hood, spice rack, pot rail and wall shelves are static, and the pedal and recycling bins
are static this pass.

The kitchen islands and bars (`models/kitchen_counters/kitchenIsland.glb`, `islandSmall.glb`,
`islandLong.glb`, `barSegment.glb`, `barEnd.glb`, same pack) are built in Blender by
`tools/art/blender/families/kitchen_counters.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
The kitchen island slides a drawer out between its stools while a resident makes a sandwich; the small and
long islands and the bars are static.

The dining tables (`models/dining/diningTable.glb`, `bistroSet.glb`, `diningGlass.glb`, `diningOak.glb`,
`tableSquare2.glb`, `tableBistroRound.glb`, `tableDining4.glb`, `tableDining6.glb`, `tableBanquet8.glb`,
`tableRound4.glb`, `tableSquare8.glb`, `tableFarmhouse6.glb`, `tableBarHigh.glb`, override pack
`packs/dining/manifest.json`) are built in Blender by `tools/art/blender/families/dining.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). They are static; the four sets keep the chairs residents sit on. The
dining table and the glass and farmhouse sets were rebuilt for two facing each other, a chair at each end of the
top, and the carver chair's seat lowered to a dining chair's (2026-10-21).

The dining chairs and stools (`models/dining/barStool.glb`, `stoolCounter.glb`, `stoolBarHigh.glb`,
`diningChair.glb`, `chairWood.glb`, `chairUpholstered.glb`, `chairModern.glb`, `chairWindsor.glb`,
`chairBentwood.glb`, `chairFolding.glb`, `chairArm.glb`, `benchDining.glb`, same pack) are built in Blender by
`tools/art/blender/families/dining.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The three
stools' seats swivel 20° and back as a resident sits or drinks, carrying them; the chairs and the bench are
static.

The dining storage (`models/dining/sideboard.glb`, `buffet.glb`, `chinaCabinet.glb`, `barCart.glb`, same
pack) is built in Blender by `tools/art/blender/families/dining.py` on `gamekit` and exported as `.glb`
(glTF, original, CC0). The bar cart's cocktail shaker lifts and tips while a resident mixes a drink; the
sideboard, buffet and china cabinet are static (their doors are parts of their own, for a later use).

The bedroom mirrors and dressing table (`models/bedroom_dressing/mirror.glb`, `dressingTable.glb`, `mirrorWall.glb`,
`mirrorFloor.glb`, override pack `packs/bedroom_dressing/manifest.json`) are built in Blender by
`tools/art/blender/families/bedroom_dressing.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Their
glass is mirror glass the game reflects the room in; the dressing table slides its drawer out while a resident
gets ready.

The bedroom nightstands and small storage (`models/bedroom_storage/nightstand.glb`, `nightstandModern.glb`,
`nightstandRound.glb`, `nightstandDrawers.glb`, `nightstandLamp.glb`, `cedarChest.glb`, `laundryHamper.glb`,
`benchBedEnd.glb`, override pack `packs/bedroom_storage/manifest.json`) are built in Blender by
`tools/art/blender/families/bedroom_storage.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The
three-drawer nightstand's top drawer slides out with a midnight snack in it, the chest's and the hamper's lids lift;
the rest are static, and the nightstand lamp's shade glows at night.

The bedroom wardrobes and drawers (`models/bedroom_storage/dresser.glb`, `wardrobe.glb`, `chestOfDrawers.glb`,
`dresserLow.glb`, `wardrobeSingle.glb`, `armoire.glb`, same pack) are built in Blender by
`tools/art/blender/families/bedroom_storage.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). Each
slides its middle drawer out (`drawer_open`) or swings its doors open onto a rail of clothes (`doors_open`,
`door_open`) while a resident picks an outfit.

The bedroom beds (`models/bedroom/bed.glb`, `bedSingle.glb`, `bedBunk.glb`, `bedFuton.glb`, `bedKing.glb`,
`bedDouble.glb`, `bedQueenUpholstered.glb`, `bedCanopy.glb`, `bedDaybed.glb`, override pack
`packs/bedroom/manifest.json`) are built in Blender by `tools/art/blender/families/bedroom.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). Each turns the top of its duvet down (`turn_down`) while a resident sleeps
or naps; the futon's back folds down flat into a bed (`unfold`) and stays down until it is folded away; the daybed is
static.

The office models (`models/office/deskSimple.glb`, `deskCorner.glb`, `deskStanding.glb`, `deskExecutive.glb`,
`computerDesk.glb`, `deskWriting.glb`, `deskLaptop.glb`, `computerGaming.glb`, `chairExecutive.glb`, `chairTask.glb`,
`officeChair.glb`, `filingCabinet.glb`, `whiteboard.glb`, `corkboard.glb`, override pack `packs/office/manifest.json`)
are built in Blender by `tools/art/blender/families/office.py` on `gamekit` and exported as `.glb` (glTF, original,
CC0). The standing and computer desks slide their keyboard shelf out while a resident types, the writing desk opens
its drawer, the laptop opens, the gaming PC's fans spin while a game is played, the desk chairs swivel as a resident
sits down and the whiteboard fills with a plan; the other desks, the filing cabinet and the corkboard are static.

The tabletop things (`models/tabletop/*.glb`: the kitchenware `kettle`, `toaster`, `blender`, `standMixer`,
`espressoMachineSmall`, `knifeBlock`, `utensilCrock`, `cuttingBoard`, `breadBin`, `spiceJars`, `cookieJar`; the
tableware `fruitBowl`, `plateSet`, `cakeStand`, `teapotSet`, `mugs`, `wineGlasses`, `candlesticks`; the books and pots
`bookStack`, `booksUpright`, `vaseFlowers`, `vaseSmall`, `potSucculent`, `potCactus`, `potSmall`; the ornaments
`photoFrame`, `tableClock`, `globe`, `snowGlobe`, `trophy`, `sculptureSmall`; and `soapSet`, `towelStack`,
`perfumeTray`, `deskOrganiser`, `laptopClosed`, `monitor`, `stuffedToy`, `toyBlocks`; override pack
`packs/tabletop/manifest.json`) are built in Blender by `tools/art/blender/families/tabletop.py` on `gamekit` and
exported as `.glb` (glTF, original, CC0). The alarm clock's hands tell the game's time and the
candles' flames rise at night; the rest are static.

The playground (`models/playground/slide.glb`, `climbingFrame.glb`, `seesaw.glb`, `springRider.glb`,
`merryGoRound.glb`, `basketballHoop.glb`, `pingPongTable.glb`, `parkChessTable.glb`, each with a `.far.glb` for the
town overview, override pack `packs/playground/manifest.json`) is built in Blender by
`tools/art/blender/families/playground.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The seesaw's
plank, the spring rider's horse and the merry-go-round's platform go round while they are played on, carrying their
riders; the hoop's net swishes after a shot and a pawn moves on the chess table; the slide, the climbing frame and the
ping-pong table are static. The chess table was rebuilt with a stool either side, its players facing each other
(2026-10-21); the slide (headroom on its deck), the spring rider, the merry-go-round, the hoop, the ping-pong table
and the chess table were rebuilt to real sizes the same day.

The farm (`models/farm/barn.glb`, `silo.glb`, `tractor.glb`, `hayBale.glb`, `hayStack.glb`, `scarecrow.glb`,
`trough.glb`, `coop.glb`, each with a `.far.glb` for the town overview, override pack `packs/farm/manifest.json`) is
built in Blender by `tools/art/blender/families/farm.py` on `gamekit` and exported as `.glb` (glTF, original, CC0).
The scarecrow sways slowly on its post near the camera (`sway`, ambient); the coop's pop-hole hatch is a part of its
own for a later use; the rest are static.

The shop fittings and café furniture (`models/shops/shopCounter.glb`, `bakeryCase.glb`, `groceryShelf.glb`,
`cafeTable.glb`, `cafeChair.glb`, each with a `.far.glb` for the town overview, override pack
`packs/shops/manifest.json`) are built in Blender by `tools/art/blender/families/shops.py` on `gamekit` and exported
as `.glb` (glTF, original, CC0). The till's cash drawer slides out while a resident buys a treat and the bakery case's
back glass door slides aside for a pastry; the grocery shelves, the café table and the bentwood café chair are static.

The food stalls and vending machines (`models/stalls/iceCreamCart.glb`, `hotDogStand.glb`, `vendingSnack.glb`,
`vendingDrink.glb`, `marketStall.glb`, each with a `.far.glb` for the town overview, override pack
`packs/stalls/manifest.json`) are built in Blender by `tools/art/blender/families/stalls.py` on `gamekit` and exported
as `.glb` (glTF, original, CC0). The ice cream cart's freezer lid slides open and the hot dog stand's hot-plate lid
lifts while a resident buys from them; a snack slides down to the snack machine's tray and a can drops to the drinks
machine's; the machines' signs and the drinks machine's front glow at night; the market stall is static.

The street furniture (`models/street_furniture/streetLamp.glb`, `parkBench.glb`, `drinkingFountain.glb`,
`litterBin.glb`, `busStop.glb`, `postBox.glb`, `noticeBoard.glb`, `phoneBooth.glb`, `fountainTown.glb`,
`townStatue.glb`, each with a `.far.glb` for the town overview, override pack `packs/street_furniture/manifest.json`)
is built in Blender by `tools/art/blender/families/street_furniture.py` on `gamekit` and exported as `.glb` (glTF,
original, CC0). The street lamp's lantern and the phone box's signs glow at night; the drinking fountain's arc of
water grows while a resident drinks, the post box's slot flap swings in for a card, the phone box's door opens and
shuts behind the caller, and the town fountain's water runs near the camera (`water_run`, ambient); the bench, the
litter bin, the bus shelter, the notice board and the founder's statue are static.

The street's dressing props (`models/street/bench.glb`, `planter.glb`, `flowerTub.glb`, `sign.glb`, `busSign.glb`,
`pot.glb`, `pathLight.glb`, `flowers.glb`, each with a `.far.glb` for the town overview, override pack
`packs/street/manifest.json`) are built in Blender by `tools/art/blender/families/street.py` on `gamekit` and exported
as `.glb` (glTF, original, CC0): the kerbside bench, the planter with its little tree, the flower tub, the street-name
and bus-stop signs, the door pot, the garden path light (its lens glows at night) and the flower
clump. All are static; their leaves and flowers sway in the wind.

The garden things and lights (`models/garden_things/pond.glb`, `planterBox.glb`, `birdhouse.glb`,
`birdFeederPole.glb`, `dogHouse.glb`, `sundial.glb`, `steppingStones.glb`, `lampPost.glb`, `stringLights.glb`, the
planter box and the lamp post each with a `.far.glb`, override pack `packs/garden_things/manifest.json`) are built in
Blender by `tools/art/blender/families/garden_things.py` on `gamekit` (the planter box's leaves and flowers with
`families/plants.py`) and exported as `.glb` (glTF, original, CC0). The pond's ripples spread near the camera and a
koi comes up to be fed or watched, a blue tit lands on the feeders while a resident watches, the lamp's panes and the
festoon bulbs glow at night and the string sways.

The ground cover, the woods' rocks and the selection marker (`models/ground/flowers.glb`, `rock.glb`,
`marker.glb`, the first two each with a `.far.glb`, override pack `packs/ground/manifest.json`) are built in
Blender by `tools/art/blender/families/ground.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The
lawn's wildflowers sway in the wind and take a tint per instance (the foliage finish). The lawn's grass is no file:
it is built on the GPU (`web/src/render/pc/grass.ts`, original code).

The living room's media (`models/media/tv.glb`, `tvWall.glb`, `tvRetro.glb`, `tvStandLow.glb`, `homeCinema.glb`,
`stereo.glb`, `boombox.glb`, `fireplaceElectric.glb`, override pack `packs/media/manifest.json`) are built in Blender
by `tools/art/blender/families/media.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). The screens glow
at night; the portable TV's rabbit ears turn and extend, the low bench's TV rises out of its lift and the home
cinema's projector screen unrolls while watched; the stereo's CD tray slides out and its woofers pulse, the boombox's
cassette door tips open, and the electric fire's flames grow while a resident warms up on its floor cushion.

The things residents carry and cook with (`models/held/*.glb`: the bag of shopping, the prep bowl, the stew pot and
the dirty pot, the tasting board, the grill platter and the dirty platter, the pizza, the baking tray and the dirty
tray, the pie and the dirty pie dish, the plates of stew, pizza, treats, pie, a sandwich and toast, the dirty plate, the
dirty tasting board and pizza board, the bowl of ice cream and the dirty bowl, the smoothie, the noodle pail and the
takeaway box, and the chopping board, the
pan, the mixing bowl and the food on the grill that stand on a station while a resident cooks; override pack
`packs/held/manifest.json`, over the `item.*` and `tool.*` keys) are built in Blender by
`tools/art/blender/families/held.py` on `gamekit` and exported as `.glb` (glTF, original, CC0). They are still.

## Generated art (original, CC0)

| File | What | How |
|---|---|---|
| `textures/sky/clouds.png` | Cloud band of the sky | `cargo tex clouds` (procedural noise, `crates/tools/src/tex.rs`) |
| `icons/rotate.svg`, `wall.svg`, `door.svg`, `eraser.svg`, `eyedropper.svg`, `environment.svg`, `fence.svg`, `gate.svg`, `roof.svg`, `upgrade.svg`, `focused.svg`, `inspired.svg`, `energized.svg`, `relaxed.svg` | UI and emotion icons | Hand-written SVG in the style of the existing set |

## Characters (`characters/`)

The residents are generated in code (original, CC0): their bodies at every life stage, faces and
their expressions and visemes, eyes, hair, beards and clothes by `cargo characters`
(`crates/tools/src/characters`; `cargo characters build` writes `rig.json` and the `.bin` files),
and their animation clips by `render/shared/characters/clips` (baked when the game loads). Bed
blankets are generated at runtime in `render/shared/characters/blanket.ts`. No downloaded models,
textures or animations are used. The facial expressions (`MOODS` in
`render/shared/characters/face.ts`) and the idle micro-expressions (`MICRO`) are adapted from the
expressions of TalkingHead by Mika Suominen (MIT, https://github.com/met4citizen/TalkingHead).

## Street and gardens (`street/`)

The neighbourhood dressing (`render/shared/street.ts`, drawn by `render/pc/street.ts`: sidewalks, kerbs, streetlights, mailboxes, hydrants, bins, cars, picket fences, flower beds; neighbour porches, chimneys and garages in `house.ts`) is built from primitives in code (original, CC0). The flower-clump models are Blender-built (`packs/street`, above); the hedges are the plants pack's (`models/plants/hedge.glb`). Only the sidewalk surface is third-party:

| File | Used for | Source asset | Authors | URL | Licence |
|---|---|---|---|---|---|
| `street/sidewalk/albedo.webp`, `normal.webp`, `arm.webp` | `material.sidewalk` | Concrete Pavement (1k; albedo made a greyscale detail map, resized to 512 px, ARM to 256 px, WebP) | Charlotte Baglioni | https://polyhaven.com/a/concrete_pavement | CC0 |

## Resident voices (`../voice/`, fetched by `cargo fetch-voice`)

Unlike the art above, these come with their own licences: the model under the **BigScience OpenRAIL-M** licence (with use restrictions), the dictionary under **Apache-2.0**. They are downloaded at build time, pinned and checked by SHA-256, and are not stored in git.

| File | Source | Authors | URL | Licence |
|---|---|---|---|---|
| `supertonic-3/*.onnxz` | Supertonic 3 (revision `3cadd1ee`): duration predictor, text encoder, vector estimator and vocoder, **modified**: every float32 weight of 1,024 values or more stored as float16 and cast back to float32 when the model loads (`crates/tools/src/fp16.rs`), then gzipped; nothing retrained (`supertonic-3/NOTICE`) | Supertone Inc. | https://huggingface.co/Supertone/supertonic-3, https://github.com/supertone-inc/supertonic | BigScience OpenRAIL-M (`supertonic-3/LICENSE.txt`) |
| `supertonic-3/tts.json`, `unicode_indexer.json`, `F1.json`…`F5.json`, `M1.json`…`M5.json` | Supertonic 3 (revision `3cadd1ee`): config, text indexer and ten voice styles, **unchanged** | Supertone Inc. | https://huggingface.co/Supertone/supertonic-3 | BigScience OpenRAIL-M (`supertonic-3/LICENSE.txt`) |
| `en-us.lexz` | Misaki US English dictionaries (`us_gold`, `us_silver`), merged and gzipped | hexgrad | https://github.com/hexgrad/misaki | Apache-2.0 (`misaki-LICENSE.txt`) |

**Use restrictions.** Supertonic 3's licence (`supertonic-3/LICENSE.txt`, shipped with it) forbids certain uses of the model and of what it makes. Among them: breaking the law; exploiting or harming minors; spreading verifiably false information to harm others, or personal information that can be used to harm someone; passing machine-made content off as human without saying it is machine-made; defaming, disparaging or harassing others; impersonating people without their consent; and discriminating against or harming people for their social behaviour, personal characteristics or legally protected categories. The full list is the licence's Attachment A. Anyone who gets the game with the model is bound by the same restrictions; the in-game Credits say so and link the licence. The voice names the game shows (Ada, Bea, Cleo, Dora, Elsie, Arlo, Ben, Cal, Dex, Ezra) are ours; the model calls its voices F1–F5 and M1–M5.

Our own code reads the text plain for the model and phonemizes it only for the mouth shapes: `crates/voice/src/g2p.rs` ports the English rules of Misaki (Apache-2.0); no espeak-ng code is used. Pitch, size and the 24 kHz resampling are done around the model in `crates/voice/src/dsp.rs` and `resample.rs` (our own TD-PSOLA and resampler); the model's starting noise comes from our own hash in `web/src/voice/supertonic.ts`.

The model runs with **ONNX Runtime Web** 1.30.0 (`onnxruntime-web`, its WebGPU build `ort-wasm-simd-threaded.asyncify.wasm`, bundled with the game), © Microsoft Corporation, MIT licence: https://github.com/microsoft/onnxruntime.

## The language model for residents' lines

Optional (Settings › Voices › What residents say › Lines › Language model), its 4-bit ONNX build, unchanged; the weights file is shipped gzipped and unpacked by the player's browser. Fetched at build time into `../llm/` (pinned and checked by SHA-256, not stored in git). Like the voices, it runs only on WebGPU.

| Model | Files | By | Source | Licence |
|---|---|---|---|---|
| LFM2.5-350M | `lfm2.5-350m/model_q4.onnx`, `model_q4.onnx_dataz` (`model_q4.onnx_data` gzipped, unpacked by the browser), `tokenizer.json`, `tokenizer_config.json` (revision `7dd49995`) | Liquid AI (model), onnx-community (export) | https://huggingface.co/onnx-community/LFM2.5-350M-ONNX | LFM Open License v1.0 (`lfm2.5-350m/LICENSE.txt`, from Liquid AI's https://huggingface.co/LiquidAI/LFM2.5-350M at revision `9e6c6ccf`: the export has no copy) |

**The LFM Open License v1.0** allows reproducing, changing and distributing the model free of charge, on conditions that include giving every recipient a copy of the licence (shipped with the model as `lfm2.5-350m/LICENSE.txt`). Commercial use by an organisation whose annual revenue is 10 million US dollars or more isn't licensed (its section 5).

The tokenizer runs with **Tokenizers.js** (`@huggingface/tokenizers`), © Hugging Face, Apache-2.0: https://github.com/huggingface/tokenizers.js. The model runs with ONNX Runtime Web's WebGPU build (`ort-wasm-simd-threaded.asyncify.wasm`).

## Typeface

**Inter** by Rasmus Andersson and the Inter Project Authors (`@fontsource-variable/inter`), SIL Open Font License 1.1: its licence ships next to the font files as `inter-LICENSE.txt`.

## Decoders (`../decoders/`, refreshed by `cargo decoders`)

Production builds ship compressed copies of the assets above (KTX2 textures, meshopt models, made by `tools/assets/optimize.mjs`; the art itself is unchanged). The game decodes the textures with this file, served from its own folder instead of a CDN; the models' meshopt geometry is decoded by meshoptimizer's decoder (Arseny Kapoulkine, MIT), bundled into the game's code:

| File(s) | What | Source | Authors | Licence |
|---|---|---|---|---|
| `basis/basis.wasm.js`, `basis/basis.wasm.wasm` | Basis Universal transcoder (KTX2 textures) | Basis Universal, PlayCanvas's build (its engine repository, at the `playcanvas` version the game uses) | Binomial LLC | Apache-2.0 |

The build tools that make the compressed copies are not shipped: the Basis Universal encoder (Apache-2.0, through ktx2-encoder, MIT), glTF-Transform (MIT), meshoptimizer (MIT) and sharp (Apache-2.0, with libvips, LGPL-3.0).
