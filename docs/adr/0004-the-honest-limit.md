# The honest limit: reject what shows more, warn what shows less

**Status:** accepted · **Date:** 2026-09-04

The boundary the flipchart does not cross by drawing. **It is ours**: it is held up by
rules we apply to the already-parsed `Graph`, not by the crate's validator. The root,
from which the whole split follows:

> **What is seen in excess is rejected. What is seen short is drawn and warned about.**

It is not decided by severity nor by how many cases there are, but by **what the user
sees**. Seeing short with a warning is honest —a `namespace` is missing, a `note`, an
icon, and it can be suspected—; seeing in excess is not honest at any price, because
the invented node **carries a label** and cannot be told apart from a real one, and
the user does not have the source in front of them to check it against.

The split has a second argument worth collecting because it is rare: **the cheap
detection is on exactly the side that has to be rejected.** The inventions are visible
from the `Graph` with no list of anything; detecting the leaks would require ninety
words of Mermaid syntax, i.e. a blacklist that grows with the language. The ethical
principle and the cheap one coincide.

## The two rules, one single rejection

Both are checked **on the `Graph`**, in the same step and moment as the style emptying
(ADR-0006). Looking at the text for syntax is what we do not want.

**Rule of asymmetry** — catches the **Phantom node**, the one *the language* invents on
seeing a loose id:

> An id is a Phantom node when it appears **only in relationships**, with no label, no
> body and no shape of its own, **and in the same diagram there is at least one node
> that has them.**

- `A --> B` on its own → **drawn.** A graph of bare ids, honest: two boxes with their
  id inside, and that is all that was said. Demanding `A[A] --> B[B]` would charge
  tokens for nothing and reject the most idiomatic Mermaid there is.
- `class Order { … }` + `Ordr --> Money` → **rejected**, with `Ordr`, its line and
  `Order` as a candidate.
- `API[API Layer] --> Db`, with `Db` appearing nowhere else → **rejected**.

**Rule of the traceable node** — catches the **Apocryphal node**, the one *the parser*
manufactures on giving up on a line it could not classify:

> Every `id` in the `Graph` has to appear as a token in the source the agent wrote,
> checked **against the source without its first line**.

Checking without the header is not a special case, it is the rule stated properly:
**the first line declares what kind of diagram this is and declares no nodes in any
family of Mermaid.**

## The traceable-node rule was measured, and it falls

Measured on 2026-09-04 against the bank of 63 cases with `flipchart check` (report 16,
harness in prototype 23). **Twelve false positives over the 42 correct cases, nine of
them this rule's**: mmdr does manufacture legitimate synthetic ids —`__start_root__`
for the `[*]` of `stateDiagram-v2`, and `journey_0`, `quadrant_0`, `packet_0`,
`treemap_0`—, and checking against the source does not tell them from an apocryphal
one. Forgiving them by the shape of the id **frees `radar-beta`**, which is the
invention the rule killed best, so the narrowing asks for exactly the list of families
the rule assumed it would not need. It buys what it promised —catches 5 of the 6
inventions, kills `radar-beta` with no list— and that is not enough to pay for it.

Of the other four false positives, one is a flat defect of the asymmetry rule:
`class X` on its own **is** a declaration and `declares_itself` does not know it.

**Open, and moved here unresolved: what to do about the six inventions.** Not the split
above, which stands. §9 of report 16 leaves the data to choose with. The rule stays in
the code until that decision is made.

## Why it was measured before it was believed

**The asymmetry of the risk is what forces measuring first: a false positive is worse
than the disease it cures.** An invention makes the user see in excess once; a false
positive makes the flipchart **never draw a whole kind of diagram** — and the agent
does not insist: it falls back to prose and does not say so.

## Size is not in here, and that is deliberate

**There is no size barrier in `show` today.** Where the limit of a readable View is, and
what is said on reaching it, is still open — and it was deliberately not settled on the
quiet inside the Honest limit: in SVG the failure at scale is **disorder, not false
relationships**, so it is a matter of quality, not of honesty. Whatever answers it will be
a barrier of its own, not a third rule here.

## Considered options

- **`parse_mermaid_strict` as the gate** — rejected, and demoted to a diagnostic. Its
  six form checks hold nothing up: over the bank of 63 cases it contributed **zero
  correct rejections and one on the wrong side** —it throws out `<<interface>>`, which
  is valid Mermaid, which mmdr draws fine, and which is the most idiomatic thing a
  class diagram has. There is no choice to make between the two, because strict is the
  validator placed *in front of* the permissive one: if the permissive one fails,
  strict fails for sure. So we enter through `parse_mermaid`, and `parse_mermaid_strict`
  runs only when it already failed, purely to get the typed `ParseError` for the message
  — the second parse is only paid where nothing is going to be drawn anyway, which is
  where latency does not matter.
- **A blacklist of Mermaid syntax** (the ninety words) — rejected. It is the traceable
  node rule read backwards: going from the words of the source to the drawing requires
  knowing *which words are syntax and therefore must not come out*, and that list grows
  with Mermaid. The rule as written goes the other way — an `id` the `Graph` claims to
  have needs no theory of syntax to ask the source whether it was there.
- **Reporting the two causes as different rejections, or one per turn** — rejected. They
  are the same rejection with two causes and are reported **together and all at once**,
  distinguished line by line because they ask for different things (*declare the id* vs
  *rewrite the line*). The message **does not apportion blame**: telling legitimate
  Mermaid from illegitimate would require the Mermaid parser as a judge, and that is
  Node and a second artifact.
- **Teaching syntax in the message** — rejected. No *"use `A[(Label)]`"*. The machinery
  exists so the agent does not tell something false, not to give Mermaid lessons.

---

*The reports and prototypes cited above by number lived in `docs/research/` until
2026-09-04, when the repo became official and they were withdrawn. The number still
identifies them: they are recovered from the git history.*
