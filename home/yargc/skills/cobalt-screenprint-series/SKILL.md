---
name: cobalt-screenprint-series
description: Generate or edit wide 16:9 surreal minimalist editorial artwork in a strict cobalt-blue, warm-ivory and black screen-print language, using the Tiny Figure, Black Staircase or Street Lamp motifs. Use when a request mentions this visual series, its 1990s experimental art-book aesthetic, cobalt pigment trails, stippled risograph texture or a variation that must preserve these palette and composition rules.
---

# cobalt screenprint series

Use this skill for raster image generation and editing. Read [references/prompts.md](references/prompts.md), choose one canonical scene, then call the image-generation tool with a composed prompt. Do not replace it with HTML, SVG, CSS or a code illustration unless the user explicitly asks for that output.

## workflow

1. Select the scene from the user's number or name. If no scene is given, use **03 — The Street Lamp**.
2. Start from that scene's canonical prompt. Preserve its subject, placement and motion pattern.
3. Apply requested changes only where they do not break the locked invariants below.
4. Generate a new image directly. For an edit, inspect the supplied image first and pass the local reference through the image tool's supported reference mechanism.
5. Return the image. Keep the accompanying text short.

## locked invariants

- Use a wide 16:9 composition with a flat graphic, two-dimensional editorial layout and large negative space.
- Use only saturated cobalt blue, warm ivory/off-white and solid black. Do not introduce grey, beige, green, red, skin tones or lighting colors.
- Keep the subject tiny, isolated and visually crisp against the vast landscape.
- Use rough screen-print or risograph texture: coarse stippling, pigment grain, visible paper fibers, uneven ink density, imperfect registration and hand-printed edges.
- Keep cobalt marks organic and hand-drawn. Use density contrast and empty areas. Never turn the movement into perfect geometric circles.
- Keep the image tactile and slightly imperfect, with no gradients, realistic lighting, photorealism, CGI, 3D rendering, polished digital finish or text.
- Retain the scene's small warm-ivory crescent moon and its specified position unless the user explicitly asks to remove it.
- Do not add buildings, vegetation, extra people, logos or decorative objects that are not in the selected scene.

For the Street Lamp scene, a radial burst is allowed only as an irregular organic concentration behind the lamp. It must not become a perfect circle or a literal glowing halo.
