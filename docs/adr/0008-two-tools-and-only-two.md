# Two tools and only two; the view id is the visible name

**Status:** accepted · **Date:** 2026-09-03

```
show(view_id: string, diagram: string)   // Mermaid; reusing a view_id replaces the View
clear(view_id?: string)                  // one View, or omit for the whole flipchart
```

There is no `update` —there is no patch: `show` over an existing `view_id` **replaces**
the View— and there is no third, query tool.

The server is called **`flipchart`**, so the host presents them qualified and a prefix in
the tool's own name buys no namespace: it duplicates it and stutters. The registered
names are therefore **`show` and `clear`**; what the host composes out of them is
ADR-0012's business.

## There is no `title`

The `view_id` **is** the visible name. A separate title would let the window say
*"Proposed structure of the orders module"* while the agent says *"the proposed one"*:
two names for one thing, which can drift apart. It also costs a fixed toll in every
conversation, including the ones that never draw, and Mermaid already brings its own
`title:` frontmatter for anyone who wants prose.

## Input validation

The only things validated before parsing:

- **`view_id`**: **prose, not a slug.** Not empty after trimming, and a cap of **64
  characters**. **No character policing** — `Current structure` is a perfectly good id.
- **`diagram`**: not empty.

## What it returns

`show`, on success — ~20 tokens, three pieces that pay for themselves:

- **The acknowledgement** confirms the `view_id` it was stored under, which is the name
  the agent is going to refer to the View by.
- **The count** is its only feedback about the drawing, because **the image never comes
  back into the context**. Severe divergence is already rejected; the count catches what
  the rules allow on purpose (the graph of bare ids) and any unforeseen future
  auto-creation.
- **The list of live Views** covers a hole that would otherwise stay open: after a
  `/clear` the conversation goes but **the flipchart survives**, so the new agent does
  not know `current` and `proposed` are still on screen and **has no way to ask**. With
  the list in the response it is told for free; without it, the way out would be a third
  tool.

It returns **nothing** about the window (whether it opened, whether it was hidden),
nothing about geometry, and **not one byte of the SVG**.

`clear` is symmetric and idempotent. **Clearing an id that does not exist is not an
error** (`isError: false`): what was asked for was that this View not be there, and it is
not. But **it is said, with the list alongside**, because a typo in the `view_id` means
the agent is telling the user about a View that is not the one on screen. Marking it as
an error would invite a blind retry over something that does not need fixing. And
`clear()` **does not close the window**: it leaves it in the *empty flipchart* state
(ADR-0009).

**Bill by use** (`cl100k_base`): ~20 tokens for a `show` that works, ~35 more per note,
~30–40 for a rejection. The **fixed toll** of the two tools, measured on 2026-09-03 over
the `tools/list` the binary emits, is **302 tokens in its worst known case** — 264 if you
discount the `$schema` that `rmcp` hangs off every schema and that says nothing — and it
is not a design criterion: what is watched is the cost per use. (The real toll is also the
host's to decide: the same tool costs +15 tokens with tool search on and +69 without it.)

## The tool description is a manual, not a channel of persuasion

It speaks to an agent that **has already decided to call**, not to one that needs
convincing — and with that bar, *when* to use the flipchart falls out of it on its own: it
moves to the project instructions line (ADR-0012), which is the only channel that works.

**The literal text lives in `tests/protocolo_mcp.rs`**, string by string, which is its
source. Why each piece is in it:

- **What it does and what it takes.** Mandatory.
- **The `view_id` with its example** (13 tokens). The only part of the description with
  measured efficacy: 17 of 17 spontaneous names came out as readable prose, not one `v1`.
- **The asymmetry clause** (34 tokens). Not there for frequency —measured, the agent never
  leaves bare ids: 0 of 17 spontaneous diagrams with a Phantom node, 0 bare ids— but
  because **the failure it prevents is silent**: after a stumble the agent does not
  insist, it falls back to prose and does not say so. There is no control without it, so
  withdrawing it would be betting with no data. **It gets revisited once the MVP exists**,
  which is when there will be real conversations to count.
- **Replacement and coexistence.** Showing again over an existing id overwrites that View
  and **is an error of nothing**: it is a failure with no possible rejection channel, so it
  has to be told up front.
- **That it dies with the session.** The agent repeats this to the user; without it, it
  promises permanence.

## Considered options

- **`update` and a patch** — rejected. `show` over an existing id replaces the View, which
  is the same operation with no second concept and no partial state to reconcile.
- **A third, query tool** ("what is on the flipchart?") — rejected. Its whole job is done
  by the list of live Views riding along in every `show` and `clear` response, for free
  and with no extra fixed toll.
- **A separate `title`** — rejected. Two names for one thing that can drift apart, plus a
  fixed toll in conversations that never draw.
- **Prefixing the tools (`flipchart_show`)** — rejected. The host already qualifies them
  with the server name; the prefix duplicates it.
- **A slug policy for `view_id`** — rejected. The id is what the user reads above the
  diagram, so it is prose; the only limits are non-empty and 64 characters.
