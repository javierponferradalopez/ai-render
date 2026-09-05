# One sheet, no index

**Status:** accepted · **Date:** 2026-09-02

The Viewer is one native window, written entirely in `egui` — no CSS and no web
technology. **One View in sight**: the front sheet, its name, two arrows `‹` `›` and
"sheet 5 of 7". **No tab bar and no list of the others.**

- **The always-visible name** is the `view_id`, which is what the agent says out loud and
  what the user types back.
- **Zoom belongs to the sheet**: each View remembers its own.
- **Fit: shrink, never grow.** Growing lies — it puts a 3-node diagram at 128 % next to a
  20-node one at 27 %.
- **Order: creation order**, which is literally the order of the stack. It is not
  reordered on `show`: with the agent moving the active sheet, reordering as well would
  leave the user with no fixed point on screen.
- **`show` leaves its View at the front** — that is turning the page. New or replaced, it
  ends up active; without this the agent says *"look at the proposed one"* and the user
  has to go find it.
- **If the agent removes the active View**, it moves to the most recent live `show`. If
  none is left, the window goes into *empty flipchart*.
- **User feedback goes through the chat.** *"go with option C"* is typed into the
  conversation, not clicked in the window. **No channel from the Viewer to the agent is
  opened.**

## The two empty states, which are mandatory

They are of different natures:

- ***Empty flipchart*** — after `clear()`. It is a **state the window stays in**, it does
  not hide: `clear()` is asked for by the agent, not by the user, so if the window
  vanished on its own the user would see a flicker with no cause and lose the position and
  size they had given it. If it is in the way, they close it, and by ADR-0010 that breaks
  nothing.
- ***Session ended*** — **a goodbye of 2–3 seconds** and the window closes itself, with the
  clock in the server thread. Leaving it on screen would turn the Ephemeral into a broken
  promise; the margin exists for the second-monitor case, so that whoever was looking finds
  out why it disappears.

## Several sessions

Two Claude Codes are two processes and **two windows**. It is not a swarm, thanks to the
deferred startup of ADR-0010. A shared window would demand a daemon, discovery and
arbitration.

## Considered options

- **Columns, a grid, and stacked sheets** — rejected, all four layouts mocked up over real
  SVG. **Comparison does not need the window**: three variants as `subgraph`s of one
  `flowchart` fit in a single View, labelled and legible, so setting N versions of a design
  side by side is the language's job. And what breaks columns is not N, it is size
  disparity: two Views come out at 73 % and 88 %, five heterogeneous ones send one down to
  20 %.
- **An index of the other sheets** — rejected. An index is an **administration control**
  over the flipchart, and the user does not administer: they observe.
- **Price accepted:** going back is **linear and blind** —from sheet 7 to sheet 2 is five
  steps, and you do not know what is on 2 until you get there. Accepted because the user
  does not navigate: they ask the agent and the page turns itself, which is one `show` and
  one turn. **The arrows stay** because they cost nothing and save the *"let me go back a
  second"* without spending a turn: it is the only control the user has, and it does not
  touch what is there, only which one is being looked at.
