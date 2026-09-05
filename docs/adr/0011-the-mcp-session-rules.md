# The MCP session rules, not the conversation

**Status:** accepted · **Date:** 2026-09-01

The flipchart's life is the MCP session's. **An MCP session is not a conversation:**
`/clear` ends the conversation and leaves the session alive, so **the flipchart outlives
it**.

The two death signals, in the order they arrive:

1. **`SIGINT`** — the first thing the host sends (`Sending SIGINT to MCP server process`).
   The binary has to **outlive it just long enough** to close its window.
2. **EOF on stdin** — what the MCP stdio specification tells the client to do.

Both are handled in the server thread, by the rule of ADR-0001.

**Price accepted:** after a `/clear` the window goes on showing the previous
conversation's diagram until the agent shows something else.

## Considered options

- **Tying the death of the flipchart to the `SessionEnd` hook** — rejected. With no
  channel, for the hook to talk to the process we would need a watched sentinel file,
  which **brings back through the back door the very channel this design eliminates**. And
  it would only exist for whoever installs the plugin, so the same product would have
  **two behaviours depending on how it was installed** — which is a worse defect than the
  stale diagram it fixes.
