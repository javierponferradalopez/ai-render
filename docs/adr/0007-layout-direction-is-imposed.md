# Layout direction is imposed, and the agent is told

**Status:** accepted · **Date:** 2026-09-02

`graph.direction = Direction::LeftRight`, imposed after the parse, **only in
`flowchart` and `classDiagram`** — and **also emptying the direction of every
`Subgraph`**, so it inherits the diagram's (or `LeftRight` if the field does not take
empty, which makes no difference because the diagram's is that one).

That it reaches the groups is not a loose end: a `subgraph` with its own `direction` is
the same knob in the agent's hands, inside the group — and **style is forbidden but
direction is decided for it**.

## Why left to right, measured

Over ninety configurations and seventeen cases. In the flagship case detours go from
1 to 0, edge deviation from 3.92× to 1.00, and the empty band of the canvas from 54 %
to 28 %. **It wins in sixteen of seventeen cases** and loses one by a detour. Zero lost
edges and zero loose boxes in either column: neither direction lies.

Outside those two families nothing is touched: forcing left-to-right is a no-op in
`sequenceDiagram` and `mindmap`, and it changes `stateDiagram-v2` and `erDiagram`
without breaking them — and those families are declared untested (ADR-0002), so they
are left as they come.

## The note, and how "did the source declare a direction?" is answered

Note (c) of the four fires **only when the source declared another direction**. If it
declared nothing, there is nothing to warn about and nothing is paid. It is needed
because **the agent is blind**: if it writes `flowchart TB` and we give it left-to-right
without saying so, it will describe a drawing that is not there.

**Implementation note:** mmdr **does not tell `flowchart TB` from `flowchart`** — the
parser initializes to `TopDown` and then overwrites. So *"did the source declare a
direction?"*, which is what decides whether to warn, is answered **by looking at the
source before handing it over**, the same way as ADR-0004's rules.

## Considered options

- **Imposing only when the agent does not declare a direction** — rejected. The flagship
  case starts with `flowchart TB`, so that route cured the sixteen class diagrams and
  left the case that matters exactly as it was.
- **Leaving the direction to the agent** — rejected. It is the one knob measured to
  actually cure the drawing, and it is the same class of decision as style: the flipchart
  decides how views look.
- **Not warning about it** — rejected. The agent is blind and would describe the diagram
  it wrote, not the one on screen.
