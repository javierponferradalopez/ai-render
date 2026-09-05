# What this product is not

**Status:** accepted · **Date:** 2026-09-02

None of this comes back as the build advances: it comes back, if it comes back, as new
effort. This ADR exists so the answer lives in one place instead of in somebody's memory.

## Of the product

Accounts, cloud, collaboration, persistence, a database, history, **export**, a public
marketplace, several simultaneous renderers, an AI of our own to generate layouts,
synchronization between machines. And **editing of the diagram by the user** — the user
observes.

## Of the surface

- **The browser as the delivery surface, and with it all of HTTP** — browser window, local
  server, port, session token, SSE, an HTML/JS viewer.
- **The bridge to MCP Apps** (`ui://` + `postMessage`), burned knowingly because a native
  window does not fit in a sandboxed iframe. It loses the free viewer in Claude Desktop,
  Claude web, Cursor and VS Code Copilot, and that is accepted because the flagship host is
  Claude Code, which does not render MCP Apps.
- **Embedded integration inside Claude Code**, if it turns out the mechanism does not exist
  today.
- **The terminal route, entire.** termaid's routing is untraceable at 3 nodes with
  inheritance and loses 28 % of the relationships at 19, and the pathology is one of
  character cells — impossible in SVG.
- **Painting the diagram in the conversation** (ASCII as a tool result). Not a matter of
  taste: there is no channel from the MCP server to the screen —`stdout` is JSON-RPC and
  `/dev/tty` fights the TUI—, so the drawing would enter the model's context, at ~393 tokens
  per render.

## Of the architecture

**The Layout Engine and the Drawing Surface as swappable pieces behind an interface** — see
ADR-0003, which is where it died and why. With it, **changing Mermaid engine** —and in
particular [merman](https://github.com/Latias94/merman)— is out **by decision, not by
ignorance**. And **fixing mmdr's groups ourselves** (its issues #140 and #136) would make us
maintainers of a layout engine.

## Of distribution

**Every install path that is not the plugin** — see ADR-0013, which carries npm, `brew`,
manual registration in other hosts, `experimental.binaries` and notarization, each with its
measurement.

## Of the life cycle

**Tying the death of the flipchart to the end of the conversation** (the `SessionEnd` hook) —
see ADR-0011.

## Of the product's scope

**flipchart writing the user's project instructions** — see ADR-0012. What it does do is
*recommend* the line from the installation documentation: advice, not a channel the plugin
controls.
