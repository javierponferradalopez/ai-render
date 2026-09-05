# Ephemeral flipchart for agents

A temporary visual channel for an AI agent to explain itself. The agent expresses
meaning —what there is and how it relates— and the system decides how it looks;
never the other way round.

## Language

### The channel

**Flipchart** — `flipchart` in code:
The temporary visual channel as a whole. It exists so the user understands what the
agent is saying, not to keep what the agent draws. It holds N Views and the Viewer
shows one at a time, with no index of the others, so the name is the metaphor
itself: a flipchart is exactly that. The one who turns the page is the agent,
because the flipchart is its channel for explaining itself; the user can go back one
page, and for anything else asks for it. Opening it is not asked: the first show
opens it, without permission and **without stealing the keyboard** — the window
comes to the front and focus stays where the user had it. What does depend on the
outside is that a first show happens at all — measured, the agent never offers it
and does not use it until asked or told.
_Avoid_: canvas, board, whiteboard; and as identifiers, `canvas`, `board` or
`whiteboard`

**Permission to open**:
The permission the flipchart was going to ask the user for before the session's
first show —the one that makes the window appear, and that back when this was
decided also stole focus—. **It does not exist, and the term is kept so that it does
not get reinvented.** Measured, it never happened: the agent announces and draws,
and waits for no yes. It was declared courtesy and not mechanism —nothing in the
protocol tells an asked-for show from one that was not— and it was withdrawn once
we saw that what it wanted to buy is already bought by that announcement, for free.
What stands in its place is an earlier consent of better quality: the window only
appears because the user asked for it, or because the user themself put in the
instruction that commands it.
_Avoid_: permission on its own, consent, opt-in, confirmation

**Ephemeral**:
That it dies with the MCP session, not with the agent's discipline. An ephemeral
artifact is not saved, is not exported and does not outlive the MCP session that
motivated it. An MCP session is **not** a conversation: `/clear` ends the
conversation and leaves the session alive, so the flipchart outlives it.
_Avoid_: temporary, volatile

**View**:
One of the N named representations that coexist on the flipchart at once —
"current" next to "proposed", or a class diagram next to a flow. It is identified by
its `id`, and showing over that `id` again replaces it. That `id` is also its
visible name: there is no second title, so the name the agent says out loud and the
one the user sees are the same.
_Avoid_: diagram, scene, tab

### The layers

**Visual Protocol**:
The subset of Mermaid the agent is allowed to write: meaning —nodes, relationships,
containment— with every id declared and nothing that asks for pixels. It is the
boundary the agent never crosses downwards.
_Avoid_: format, payload, DSL, and "Mermaid" on its own — the whole language is not
the protocol

**VisualDocument**:
The complete semantic state of a View, written in the Visual Protocol and kept as
is. It is the truth of the View, not a copy of something earlier.
_Avoid_: model, scene, graph, source

**Layout Engine**:
The piece that decides where everything goes: it takes a VisualDocument and produces
a PositionedScene. Neither the agent nor the user takes part in that decision, and
it is not swappable — it comes bundled with the language, in the same piece that
understands it.
_Avoid_: positioning engine, autolayout

**PositionedScene**:
A VisualDocument with geometry already resolved: what is left once the Layout Engine
has done its work and before a single pixel is painted. It lives inside the MCP
server and does not cross over to the Viewer.
_Avoid_: layout, positioned scene

**Drawing Surface**:
What the picture is finally painted with inside the viewer. What it is made of is
invisible to everything upstream, and it does not contradict the Visual Protocol's
prohibition: what is forbidden is that the **agent** produces HTML or drawing
characters, not that they exist.
_Avoid_: renderer, canvas, painter

**Renderer**:
A contaminated term: it blurs the Layout Engine with the Drawing Surface, which are
distinct stages even though today they arrive in the same piece. Name the one you
mean.
_Avoid_: using the word on its own

**Honest limit** — `honest_limit.rs` in code:
The boundary the flipchart does not cross by drawing: past that point it does not
draw worse, it stops and says so. It covers the View size that can no longer be
read, and also the meaning that does not hold up — drawing a relationship that does
not exist is worse than drawing nothing. From there comes where it runs: **what is
seen in excess is rejected; what is seen short is drawn and warned about**. And it
is ours, not the language's nor the drawer's: it is held up by the rules we apply to
what has already been parsed —the Phantom node and the Apocryphal node—, having been
measured as the renderer's and turned out to hold nothing up.
_Avoid_: limit on its own, truncation, degradation

**Phantom node**:
The node the language invents on seeing an id that only appears in a relationship,
while the others in the same diagram do carry a label or a body. What deceives is
the asymmetry: it comes out empty next to a full one, so it does not read as the
error it is but as something we know less about. A whole diagram of bare ids has no
phantoms —it promises nothing it does not deliver—, and the Honest limit exists to
reject the ones that do. It is one of the two causes of the same rejection, and the
one that gives itself away; the other is the Apocryphal node.
_Avoid_: implicit node, auto-creation, typo

**Apocryphal node**:
The node whose id is nowhere in the diagram the agent wrote: it is manufactured by
whoever parses, on giving up on a line they could not classify, and attributed to
someone who did not write it. It is the Phantom node's sibling and the worse of the
two, because **it carries a label**: the phantom shows by what it lacks, and this
one lacks nothing. Both are rejected alike and counted together, but they do not ask
for the same thing — the phantom is fixed by declaring the id, the apocryphal one by
rewriting the line.
_Avoid_: fallback node, made-up node, hallucination, garbage

**Literal markup**:
The markup the agent writes inside a label's text and that reaches the drawing
exactly as written: `<b>recolocacion</b>` in the box, angle brackets and all. It is
the fourth thing the flipchart warns about, and the only one that does not travel in
an IR field but inside the label, so emptying the style does not touch it. Its
boundary is not a policy of ours but what mmdr can interpret, and that is exactly
two strings —`<br>` and `<br/>`—: those coexist without a warning because they do
what the agent wanted, and everything else shaped like markup —tags, `&…;` entities
and `#…;` escapes— is drawn and warned about. It is warned about and not rejected
because it is seeing **one word** in excess, not a node: the structure does not lie,
and throwing away the whole drawing would charge for the explanation over a defect
of text.
_Avoid_: HTML on its own, garbage, markup without qualification

### The live pieces

**MCP server**:
What exposes the tools to the agent and is the **owner of the state** of the
flipchart. The truth lives here. It is not a process of its own: it shares a process
with the Viewer and lives in a different thread from it.
_Avoid_: backend, host

**Viewer** — `viewer.rs` in code:
What receives a scene and paints it, in its own window: one sheet in sight, titled
with the id of its View, and the one the agent has just shown at the front. It is
dumb by design: it keeps nothing, and closing its window loses nothing because the
state is not its own.
It does not restart: its window is hidden and shown again within the same process,
which only dies with the MCP session.
_Avoid_: frontend, client, app, page

**Delivery surface**:
How the viewer reaches the user's eyes. Distinct from the Drawing Surface, which is
what the painting is done with inside.
_Avoid_: surface on its own

**Launcher** — `launcher.sh` in code:
What the host invokes so that the Flipchart process exists. **It does not bring it**:
the executable is already on the machine when the launcher runs. But it does leave
it usable —it may arrive without execute permission, and granting it is the
launcher's job—, because nobody promises what state it turns up in. And it never
fails, because its failure does not look like an error but like a flipchart that
stops existing without saying why; when it cannot hand over its place, it stays and
talks itself, as the Unavailable server.
_Avoid_: installer, wrapper, shim, startup script

**Unavailable server**:
The Launcher's face when there is no Flipchart process to hand its place over to. It
says just enough to state that the flipchart is not operational and what was found;
it does not draw, keeps nothing and does not try to fix itself. It exists because
silence is the only failure this product cannot afford: there is nobody else who can
tell the user. It takes its name from the one tool it announces, `unavailable`,
which is the whole of its surface.
_Avoid_: stub, fallback, degraded mode, server on its own

**Flipchart process**:
The only process there is, and which is both MCP server and Viewer. The host
launches it as a child and talks to it over stdio. It splits the two roles between
threads: the main one draws, the secondary one serves. Which one is in charge is not
symmetric — the thread that serves is the one that knows what time it is and the one
that decides when the process exits, because the one that draws freezes when the
system covers the window.
_Avoid_: daemon, server on its own
