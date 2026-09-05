# One layout engine, pinned and not swappable

**Status:** accepted · **Date:** 2026-09-02

Parse, layout and SVG all come from **`mermaid-rs-renderer` (mmdr) `=0.3.1`**, MIT,
`default-features = false`. One engine, one exact version, and no interface behind
which a second one could be plugged.

## The version is exact, and it is raised by hand looking at the pictures

In a layout engine a patch changes the drawing, and the project buys two properties
from it: **determinism byte for byte** (same source → same SVG) and **stability on
update**. Measured: **adding one class to a diagram of six moved a node**. We give up
free layout fixes in exchange for the drawing not moving behind our backs. `=0.3.1`
is a decision, not an oversight.

Risk taken and written down: version `0.3.1`, **a single maintainer**, and a warning
from the author himself that visual quality *"may not yet match mermaid-cli"*.
Mitigations, not solutions: exact version, MIT license so we can fork, and the typed
`ParseError` its author wrote for *"LLM correction loops"*.

## The factory knobs, untouched

`Theme::mermaid_default()` (16 px font — not `Theme::modern()`, which is 14 px) and
`LayoutConfig::default()`.

**No mmdr knob improves the drawing and several do harm**, measured over ninety
configurations: `preferredAspectRatio` reorders nothing —it only stretches the canvas
until the requested ratio is met, inflating groups with empty space: asking for 3.0
takes the detour up to 10.70×—; `nodeSpacing`, `rankSpacing` and the paddings move
±60 px of canvas with the same routing; the five themes only change color, except for
the font size, and at 10 px the drawing gets worse. And **half of
`FlowchartLayoutConfig` is not even wired up**: `objective.*` and `routing.*` in full
do not change a single byte of the layout. Only `order_passes`, `port_side_bias` and
`auto_spacing` move anything, and none of them cures.

`LayoutConfig` and `Theme` are `Serialize + Deserialize`, so if there ever are values
of our own they are written as data and merged over the default.

## The groups: accepted with a ceiling, and no plan B

The bar is not *"is it as good as Mermaid.js?"* —that is out of scope— but *can a
refactor be read without following edges with your finger?*. With left-to-right
imposed (ADR-0007) the flagship case meets the three criteria: no edge crosses an
empty region, the layers come out in an order you can count, and it fits in one
window.

**Price accepted:** left-to-right straightens the routing by tipping the drawing over.
The flagship case lands at **5.23:1**, and with the *shrink, never grow* rule on a
1200×800 sheet its zoom drops from 72 % to 56 %. A small drawing you can follow with
your eyes beats a big one where you have to follow an edge with your finger.

## Considered options

- **merman** — rejected, by decision and not by ignorance. It exists, is
  MIT/Apache-2.0, declares parity with `mermaid@11.17.2` and Zed uses it. The MVP has
  one engine, and measuring a second one opens a door it does not need. It comes back
  if mmdr is abandoned, or if the MVP exists and the groups still hurt.
- **Fixing mmdr's groups ourselves** (its issues #140 and #136) — rejected. It would
  turn us into maintainers of a layout engine.
- **The Layout Engine and the Drawing Surface as swappable pieces behind an
  interface** — rejected. It was the architectural heart of the starting material and
  survived three turns; it dies for **lack of a candidate**. They remain two *stages*
  and the glossary keeps them as terms; what is closed is treating them as
  interchangeable.
- **A version range instead of `=0.3.1`** — rejected. A patch in a layout engine
  changes the picture, and the change would arrive without anyone looking at it.
