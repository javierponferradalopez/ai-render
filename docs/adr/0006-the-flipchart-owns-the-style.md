# The flipchart owns the style, and says what it dropped

**Status:** accepted · **Date:** 2026-09-04

**The style belongs to the flipchart, not to the agent.** It is emptied **from the IR
after the parse** — it is not chased in the text, because however many new ways of
writing style Mermaid invents, to have any effect they have to land in one of these
nine channels:

| | |
|---|---|
| `class_defs` | `classDef` |
| `node_classes` | `class`, `:::` |
| `node_styles` | `style` |
| `subgraph_styles` | |
| `subgraph_classes` | |
| `edge_styles` | `linkStyle` |
| `edge_style_default` | |
| `node_links` | `click` (which is also interaction, and out of scope) |
| `init_config` | `%%{init: …}%%` (it is on the `ParseOutput`, not on the `Graph`) |

**It is warned about when they came in full** — cleaning up silently would reintroduce
the lie from our side: an agent that wrote `classDef danger fill:#f00` would tell the
user *"the risky modules are in red"* and there is nothing red on screen. The common
rule for the four notes is that **you warn for what came in, not for what took effect**:
a `classDef` no class was using also warns, because the agent believed it was painting.

Written down as residue: **the pixel prohibition is cleaned-and-warned, not
impossible.** Nothing structural stops a *new* field of a future version slipping
through; with the version pinned at `=0.3.1`, that shows up when the version is raised
and not before.

## The four notes, which are not rejections

The View **is drawn** and a note is attached, with `isError: false`. All four are
**fixed literals**, so their cost is predictable, and they accumulate.

**(a) Style dropped** — one single note covers everything (style, `click` and
`%%{init}%%`), not one per category. ~35 tokens.

**(b) Structure mmdr does not draw** — only `namespace` and `note`, and **only in
`classDiagram`**. It says something different from the previous one: the style note says
*we decide how it looks*, this one says *we do not know how to draw this*. They are
written as what they are: **two of mmdr's debts, with a name and a date, not a policy.**
They are in because `classDiagram` is the flagship case and `namespace` is how a class
diagram says *module* — and the refactor we want understood is a move between modules.
Two text checks, and **note only, never rejection**: rejecting would leave the agent
unable to draw a valid class diagram because of a limitation of ours.

**(c) Direction imposed** — ADR-0007.

**(d) Literal markup** — the markup the agent writes **inside** the label and that
reaches the drawing as text. It says the same as (b) —we do not know how to draw it—
about the one thing the emptying above cannot touch, because it travels in the text and
not in a field. **39 tokens**, and the most expensive of the four, and the one that buys
most, because it is **the only channel that exists to fix this rubbish**: the agent is
blind and the user does not read the source, so without it `<b>recolocacion</b>` stays
in the box forever. Measured, it is paid in 5 of 17 spontaneous diagrams.

## The split of the markup is per construct, not wholesale

Measured on 2026-09-04 with the drawing in front of us (report 17, bank in prototype 24),
sweeping thirty cases in `flowchart` and `classDiagram`:

| Construct | What mmdr does | Outcome |
|---|---|---|
| `<br>`, `<br/>` | **interprets it** — splits the label | **coexistence, no note**: it is what the agent wanted |
| everything else | **escapes it** | **note (d)**, and the View is drawn |

"Everything else" is the rest of the tags (`<b>`, `<i>`, `<em>`, `<strong>`, `<u>`,
`<code>`, `<span>`, `<a>`, `<img>`), the entities (`&amp;`, `&nbsp;`, `&lt;`, `&#35;`),
Mermaid's own escapes (`#quot;`, `#35;`) and **even `<br />` with a space inside, `<br  />`
and `<BR/>`**. The boundary is mmdr's implementation, not a policy of ours.

It is asked of **the `Graph`'s labels**, not of the source —same as the rules of
ADR-0004—, which is where there is no Mermaid syntax left to confuse with markup: the `&`
of `A & B` is not an entity and `-->` is not a label. Three shapes count as markup: a tag
`<name …>` that is not `<br>`, an entity `&…;` and a Mermaid `#…;` escape. Two that look
like it do **not** count: `<<interface>>`, which mmdr draws fine, and `Map<String,Int>`,
where the comma gives away that there is no tag name. Over the bank of 63 plus the
families and probes —74 cases— **the note fires on one, and that one carries `<b>`**.

The numbers on the scale: `<br>` in **15 of 17** spontaneous diagrams, and **5 of 17**
with visible rubbish today (`<b>` in three, `<i>` in one, `&lt;`/`&gt;` in one). So the
note is paid in 5 of 17, not 15 of 17.

## What is lost without a note, written down

None of this is a bug to fix: it is known cost.

- **Layout hints**: `A ----> B` and `A --> B` are the same byte — the IR's `Edge` has no
  length field.
- **`cssClass` and `link` of `classDiagram`**: style with no field to look at. They are
  discarded at parse time and land nowhere, so there is nothing to warn about.
- **Five structure leaks** in untested families: `note for`, loose prose in
  `classDiagram`, titles of `C4Context` and `zenuml`, icons of `architecture-beta`.
- **Two deformations**: literal markdown and `zenuml` draw ugly. Coexistence, no note.
- **The literal-markup note does not reach the side channels**: it looks at the labels of
  nodes, groups and edges, so a `Note over` in a `sequenceDiagram` with `<b>` inside is
  drawn literal and silent. The two families that are promised do not have that hole.

## Considered options

- **Cleaning up silently** — rejected. It reintroduces from our side the very lie the
  product exists to refuse: the agent describes colors that are not on screen.
- **Chasing the style in the text** — rejected. It grows with every new way Mermaid
  invents of writing style; the nine IR fields are where all of them have to land to have
  any effect.
- **Rejecting literal markup, as a sixth outcome** — rejected. It did not fit the table of
  five (ADR-0005) without becoming a sixth outcome, and above all it **charges for the
  whole structure over a defect of text**: throwing away a well-written dependency diagram
  because one box says `<b>` trades seeing one ugly word for seeing nothing — and the
  agent, measured, does not insist after a stumble, it falls back to prose and does not
  say so.
- **Coexisting with literal markup without a note** — rejected. It is the only channel
  that exists to fix it; with the note, permanent rubbish becomes one turn's rubbish.
- **Charging for `<br>` too** (rejecting it or noting it) — rejected. It comes out right,
  and charging for something that works is charging for nothing.

---

*The reports and prototypes cited above by number lived in `docs/research/` until
2026-09-04, when the repo became official and they were withdrawn. The number still
identifies them: they are recovered from the git history.*
