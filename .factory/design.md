# Visual thesis — The proof orbit

## Direction and rationale

The direction is **generative geometry**: a calendar is a mathematical promise,
while a receipt is the point that proves reality met that promise. Repeating
orbits, plotted nodes, and one conspicuous gap make absence legible before any
copy is read. The system feels like an operator's evidence instrument rather
than a generic SaaS dashboard.

The site is intentionally single-mode, painted in midnight ink. It matches the
quiet, always-on world of scheduled operations and lets the warm evidence marks
carry the hierarchy. The exported report uses an ink-on-paper light treatment
because it is meant to print, attach, and archive.

## Palette

| Token | Value | Role |
| --- | --- | --- |
| Ink | `#071A23` | Explicit page background |
| Raised ink | `#102A35` | Code, controls, grouped evidence |
| Chalk | `#F4F0E6` | Primary text and expected nodes |
| Steel | `#9DB3BC` | Secondary text (≥ 7:1 on ink) |
| Vermilion | `#FF664A` | Missing/failed and primary action |
| Deep vermilion | `#8F2415` | Accessible light-surface danger text |
| Mint | `#7FE0B2` | Verified success |
| Amber | `#F6C86B` | Late/running warnings |
| Hairline | `#31505B` | Dividers and orbit construction lines |

State is never color-only: every mark pairs color with a word, symbol, or
shape. Focus uses a 3 px chalk/vermilion double ring.

## Typography

- Display and body: `Instrument Sans`, self-hosted variable WOFF2, with the
  system sans stack as fallback. Its open counters and engineered curves fit
  the plotting language without becoming sterile.
- Code and evidence numerals: `IBM Plex Mono`, self-hosted WOFF2, with the
  system monospace stack as fallback. Tabular figures keep time columns stable.
- Scale: 14, 16, 20, 28, 44, and fluid 72 px; body is never below 16 px.
- Reading measures stay between 48 and 72 characters.

## Spacing and geometry

An 8 px base rhythm drives spacing (`4, 8, 16, 24, 32, 48, 72, 96`). The site
uses a 12-column grid above 900 px and a single deliberate stack at 390 px.
Corners are clipped rather than rounded: 12 px diagonal cuts echo receipt
stubs and locator brackets. Cards appear only for independent evidence units.
Touch targets are at least 44 px with 8 px between neighbors.

## Interaction grammar

- Primary actions fill with vermilion; secondary actions are chalk outlines.
- The live evidence strip is a seven-slot keyboard-operable radio group.
  Selecting a slot swaps its receipt state and the text summary together.
- Copy buttons confirm in-place using a short label change announced through a
  polite live region.
- On mobile, decorative construction lines disappear and the evidence strip
  becomes a compact 4 + 3 grid; no operational content is removed.

## Motion policy

On entry, receipt nodes resolve once along their orbit over 480 ms, while the
missing-slot locator settles over 240 ms. State changes use only opacity and
transform for 180 ms. Nothing loops. Under `prefers-reduced-motion: reduce`,
all transforms and smooth scrolling are removed and state changes are instant.

## Asset plan and provenance

The hero illustration is an original generated raster,
`site/public/assets/receipt-orbits.webp`, produced 2026-08-28 with the factory's
`factory-image` deployment through `/opt/fleet/lib/gen-image.sh`. Prompt:

> A wide editorial computational-art hero for a local-first scheduled-run CLI:
> seven precise circular time-orbits on near-black navy, six ivory receipt
> nodes forming a trail, one deliberate empty slot caught by a vermilion
> locator bracket, and a mint confirmation seal; crisp plotted geometry,
> restrained screenprint grain, negative space on the left; no text, logo,
> dashboard, UI screenshot, gradient, or watermark.

The source PNG and generator metadata are retained beside the optimized WebP.
UI icons and small timeline marks are original CSS/SVG geometry authored in
the repository; they use no third-party icon library. Generated asset rights
follow the factory's OpenAI output terms; code-authored assets are MIT.
