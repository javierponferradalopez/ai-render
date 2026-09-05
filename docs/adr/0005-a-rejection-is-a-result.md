# A rejection is a result, not a transport error

**Status:** accepted · **Date:** 2026-09-02

Every rejection travels **inside the tool result, with `isError: true`**, never as a
JSON-RPC error.

- A `ParseError` designed for correction loops, returned as a transport error,
  **throws away the line, the column and the candidates**, and the agent retries
  blind or gives up.
- And `isError: true` is what separates *"nothing was drawn"* from *"it was drawn"*.
  A rejection without the mark reads as success, and then the agent describes to the
  user a diagram that is not on screen — **the same lie this product exists not to
  draw, one level up**.

**A rejection does not touch the flipchart.** If `show("proposed", …)` fails and there
already was a `proposed` View, the old one stays intact and on screen: it is not
cleared and not replaced by a gap.

## The five outcomes of `show`

| # | Outcome | `isError` | Drawn? |
|---|---|---|---|
| 0 | Invalid input — `view_id` empty or >64, or `diagram` empty | `true` | no |
| 1 | mmdr `ParseError`, including the unknown variant | `true` | no |
| 2 | Phantom node or Apocryphal node — one or several, all reported | `true` | no |
| 3 | Renderer panic | `true` | no |
| 4 | Success, with or without a note | `false` | yes |

The order is **complete** —any `show` lands in one of the five rows, and in all five
the agent knows whether there is something new in the window— and **1 comes before 2
of necessity**: the rules are checked on the `Graph`, and if the parse failed there is
no `Graph`.

Every rejection carries **two pieces**, in English, because it is text for the model:

1. **A fixed first line saying what did not happen**, always the same —
   `Rejected: nothing was drawn; view "<id>" is unchanged.` It is what stops the agent
   from going on talking about a drawing that does not exist.
2. **The diagnostic, one line**, with whatever the case brings: id, line, column,
   found/expected, candidates. If `expected` is a long list, the first three.

**The source line is not echoed back**, except inside the `ParseError` (where it comes
in the pass-through and taking it apart would cost more than it saves): it would be
charging the agent for handing back something it just wrote and has in front of it in
the same turn. **The full source is never returned.**

**Outcome 0 — invalid input.** Rejected **at the door, without consulting mmdr**: it is
the one place where we validate before parsing, and that is because there is nothing to
parse.

**Outcome 1 — `ParseError`, with a hybrid pass-through.** Our own text for the variants
we care about (`UnknownParticipant { name, line, candidates }`, `UnclosedSubgraph
{ opened_at }`, `UnexpectedToken { line, col, found, expected }`) and **mmdr's `Display`
as the filler of the wildcard arm**, preceded by an admission that we have not
classified it. With the version pinned, the wildcard arm of `#[non_exhaustive]` is a
**safety net, not a maintenance treadmill**.

**Outcome 3 — the panic, and the only one that says the fault is ours**, on purpose: if
we tell the agent to fix its diagram, it will try in a loop on something that has no
fix. Steps 1, 5 and 6 of the pipeline are wrapped in a panic guard (`catch_unwind`) —
server and Viewer share a process, so an uncaught panic would take the whole flipchart
down, silently.

**A rejection never carries notes.** If nothing was drawn, also telling the agent we
dropped its colors is noise about something it is going to rewrite whole.

## Who all of this is for

The user **reads none** of what `show` and `clear` return, and should not — the
server's only channel to their eyes is the window. What the user ends up seeing is the
agent talking. Hence the machinery splits into two pieces with different owners:

- **That the agent does not lie** is bought by `isError: true` **plus the fixed line**,
  and nothing else.
- **That the agent fixes it and comes back** is what the diagnostic pays for, and it is
  not hypothetical: measured, when the flipchart does not work out the agent **does not
  insist — it falls back to ASCII or prose, and does not say so**. A rejection without
  clues does not produce a retry: it produces an explanation in text and a user who
  will never know there was a flipchart.

## Considered options

- **The JSON-RPC error** — rejected. It is the obvious home for something called an
  error, and it throws away the line, the column and the candidates, which is the only
  part that produces a retry.
- **The rejection without `isError`** — rejected. It reads as success, and the agent
  then describes a drawing that is not there.
- **Our own text for all twenty `ParseError` variants** — rejected, as is the bare
  `Display`, which is what its author wrote the enum to avoid. The hybrid covers the
  variants that matter and admits it when it does not know.
