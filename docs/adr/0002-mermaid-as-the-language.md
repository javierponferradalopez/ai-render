# Mermaid as the language, not a protocol of our own

**Status:** accepted · **Date:** 2026-09-02

**The agent writes Mermaid.** There is no protocol of our own, no primitives, no
`kind`, no patch. The Visual Protocol is a *subset* of Mermaid —meaning, with every
id declared and nothing that asks for pixels—, not a format we invent.

A semantic protocol of our own would be implemented over Mermaid anyway —every entry
door of the crate starts with a `&str`— so the only thing it would decide is *who
types that text*. And it would cost **738 tokens of fixed toll against 271**, with
**break-even at 4.8 touch-ups** on the same View: only an agent that revisits a
single View five times or more comes out ahead.

The independence argument turns around, too: Mermaid is read by Mermaid.js, mmdr,
mermaid-cli, Kroki, GitHub, GitLab and the Cursor CLI — **seven implementations
against one**. Our JSON would be read by exactly one.

## What survives of the protocol idea

- **The prohibition** — no HTML, SVG, CSS, coordinates, sizes, colors or concrete
  shapes. It is defended by emptying the IR and warning (ADR-0006), not by the
  grammar.
- **The two honesty rules** (ADR-0004).

## What is promised of the language

**One measured family: the directed graph.** `architecture`, `flow`,
`dependency-graph` and `class-diagram` are the same engine, and they are the bar for
what is promised — and also the scope where direction is imposed (ADR-0007).

**The other families are not forbidden: they are untested.** With Mermaid as the
language they draw themselves, and forbidding them would cost writing code to forbid
them. They have been probed **once**: none of the 23 comes out empty; `radar-beta` is
not really implemented, `C4Context` loses its title, `architecture-beta` loses its
icons. `wireframe` is the one real exception — Mermaid does not have it.

Worth keeping in mind: the agent picks unmeasured families on its own
(`sequenceDiagram`, 4 of 17 spontaneous diagrams).

## Considered options

- **A semantic protocol of our own (primitives, `kind`, patches)** — rejected. 738
  tokens against 271, break-even at 4.8 touch-ups on the same View, and it would run
  on top of Mermaid regardless. What it buys is deciding who types the text.
- **Independence from Mermaid as an argument for the protocol** — rejected because
  it points the other way. The dependency is on a language seven implementations
  read; the protocol's dependency would be on the one implementation that is ours.
- **Forbidding the untested families** — rejected. They draw themselves for free;
  forbidding them means writing and maintaining code whose only effect is to say no.
