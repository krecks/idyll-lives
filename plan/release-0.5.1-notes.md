**Play it in your browser:** https://krecks.github.io/idyll-lives/

Three fixes: the town seen from the menus, how smoothly it moves, and residents going next door.

- **Streets in the menus' town:** no more green patches on the roads. From the overview's height the graphics card couldn't tell the roads from the ground just beneath them; the camera now keeps its precision for where it is looking, near or far. The game's own wide view gains a cleaner edge to the side roads too.
- **Frame rate in the menus:** the town behind the menus draws at the frame rate cap from Settings (30, 60, 120 or none), as the game does. Before, it kept to 30 frames a second, and its camera juddered as it glided in or was dragged. Standing still it still costs nothing: the last picture is shown until something changes.
- **Next door by the street:** residents going into a neighbour's garden no longer walk through the hedge or fence between the gardens. They go out to the street and in at the front.

### Also
- **Saves:** unchanged (format 33): 0.5.0 and 0.5.1 open each other's saves.
- **Known:** choosing another world on the new-world screen still takes a tenth of a second while the world is made, as before.
