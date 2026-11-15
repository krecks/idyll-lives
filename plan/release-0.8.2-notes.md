**Play it in your browser:** https://krecks.github.io/idyll-lives/

A fix to 0.8.1's sun shadows on WebGPU.

- **Shadows where they belong on WebGPU:** since 0.8.1 the sun's shadows of everything that stands still (houses, trees, fences) came out mirrored on WebGPU, lying across the lawn away from what cast them, while the residents' were right. The kept picture of the still casters was drawn the other way up from the sun's map and copied in so; it is drawn the same way up now. WebGL2 was not affected.

Saves are format 39, as in 0.8.1.
