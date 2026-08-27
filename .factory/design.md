# Capacity Sentinel — visual thesis

## Direction: a botanical field guide for model ecosystems

Capacity Sentinel treats each model endpoint as a living specimen: observed at regular intervals, annotated with evidence, and compared by the same field method. This fits the product because it is neither an AI chat surface nor a generic observability console. The interface should feel like a careful naturalist's ledger—calm enough for daily use, precise enough for an incident.

The UI uses one explicit light treatment, “pressed paper at dawn.” Monitoring rooms already contain enough dark dashboards; this warm light canvas improves label legibility and makes warning inks unmistakable. A dark mode is intentionally omitted because the thesis is materially tied to paper, graphite, and specimen ink.

## Palette

| Token | Hex | Use |
|---|---:|---|
| `paper` | `#F3F0E5` | page background, field-guide paper |
| `paper-raised` | `#FBFAF4` | forms and active specimen sheets |
| `ink` | `#17231B` | primary text (13.8:1 on paper) |
| `ink-muted` | `#526257` | annotations (5.7:1 on paper) |
| `moss` | `#315B3A` | primary action and healthy state |
| `moss-deep` | `#20442B` | pressed/hover state |
| `lichen` | `#CCD8B9` | low-emphasis healthy wash |
| `ochre` | `#956315` | latency warning and annotations |
| `rust` | `#A33B2B` | errors, SLO breach, destructive action |
| `rule` | `#C8C7B9` | ledger rules and dividers |
| `focus` | `#175F84` | keyboard focus ring |

Status never relies on colour alone: every state has a plain-language label and a distinct leaf/triangle/cross glyph.

## Typography

- Display and section labels: Georgia, `Times New Roman`, serif. It evokes reference books without downloading a font and gives model names a specimen-label character.
- Interface, forms, and data: system sans (`Inter`-compatible native stack). Tables use tabular figures. No third-party fonts or runtime assets.
- Scale: 14 annotation, 16 body, 20 section, 28 panel title, clamp(40–68) page title. Body leading is 1.55 and long text is capped at 68 characters.

## Spacing and form

An 8px rhythm with 4px for micro-gaps: 4, 8, 12, 16, 24, 32, 48, 64. Content sits in a 1240px field sheet with an offset 12-column grid. Rules, margin notes, specimen numbers, and clipped-corner panels create hierarchy; rounded “SaaS cards” and ornamental gradients are avoided. Controls are at least 44px high. On 390px screens the margin notes disappear, the nav becomes a compact horizontal trail, and tables become labelled observation blocks rather than squeezed columns.

## Interaction grammar

- “Observe now” is the primary verb and uses a filled moss button.
- Opening a specimen preserves spatial continuity: details expand directly beneath its row.
- Fresh observations briefly receive a pale lichen wash; alerts are pinned with a rust margin mark.
- Loading uses a single moving scan line across the active specimen. Empty and offline states show a useful next action, never a dead end.
- Keyboard order follows the reading order; focus is a 3px blue field-pencil outline with 2px clearance.

## Motion policy

UI transitions last 180–240ms and animate only opacity and transform. The hero’s tiny observation markers enter once in a staggered 300ms reveal; nothing loops. Under `prefers-reduced-motion: reduce`, all transforms and smooth scrolling are disabled and state changes are instantaneous. Depth remains through borders, overlap, scale, and ink density.

## Asset plan and provenance

The hero uses one original raster illustration: a night-green model “habitat” rendered as a scientific cyanotype/botanical plate, where branching stems connect glass specimen nodes and one ochre leaf signals a degraded endpoint. It explains the central metaphor and creates a distinct product world. UI icons and charts are hand-authored SVG/CSS because they must remain crisp and semantic.

### Prompt sheet

- Subject: abstract botanical network used to observe model endpoints; branching stems, glass seed pods, measurement ticks, one stressed ochre leaf.
- World/materials: archival field-guide print, cyanotype ink, pressed fibers, engraved linework, translucent laboratory glass.
- Light/lens: flat scientific plate with subtle raking light; orthographic, no photographic depth blur.
- Palette words: deep moss, warm paper, lichen, oxidized ochre, graphite.
- Composition: wide landscape, main structure on the right, calm negative space on the left for UI copy; no border.
- Negative list: no text, letters, numbers, watermark, logo, people, faces, hands, brand marks, neon gradients, glossy 3D, generic circuit brain.

Final generation prompt: “Wide editorial hero illustration for a model API capacity monitor. An abstract botanical network specimen: branching deep-moss stems connect translucent glass seed pods like monitored endpoints; delicate engraved measurement ticks and small field-note marks; one oxidized-ochre leaf subtly droops to signal degradation while the remaining lichen leaves are healthy. Archival botanical field-guide plate mixed with restrained cyanotype and scientific engraving, warm pressed-paper fibers, flat orthographic composition, subtle raking light. Place the botanical structure primarily on the right with calm low-detail negative space on the left. Deep moss, warm paper, lichen green, graphite, oxidized ochre. No text, no letters, no numbers, no watermark, no logos, no people, no brands, no neon gradient, no generic circuit brain.”

Generated with the factory `factory-image` deployment on 2026-08-27. Original to this product; no third-party source material. Source PNG and prompt sidecar live in `assets/src/`; optimized WebP is shipped in the frontend.
