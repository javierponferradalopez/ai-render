# One process, two threads, no IPC

**Status:** accepted · **Date:** 2026-09-01

The host launches the binary as a child process and talks to it over stdio. Inside
there is one process and two threads:

- **Main thread: the `winit`/`egui` event loop** — where macOS demands it be. It only
  draws.
- **Secondary thread: the MCP server** — owner of the state of the N Views, and the
  one that decides when the process exits.

With that the whole of IPC disappears: no socket, no internal protocol, no
handshake, no orphaned processes. And being Ephemeral stops being a rule somebody
applies: it is the life of the process, guaranteed by the operating system.

## The rule that governs the split: the event loop is not a clock

**With the window fully covered by another one, macOS does not slow the event loop
down: it stops it.** Measured: four `show` calls in a row without a single repaint in
12 s, and a dead session the process took 11 s to notice, because the notice is
handled in `update()` and there was no `update()`.

That it does not repaint while nobody is looking is not a defect —uncovering the
window draws the latest state—. The defect is the other side: **processes that
outlive their session**, with a phantom flipchart showing the diagram of a dead
conversation. And covered is the *normal* case: the user is in their terminal.

> **Anything that depends on time or on the death of the session lives in the server
> thread, which does keep running. The server thread has the last word on the exit of
> the process; the event loop only draws.**

`beginActivityWithOptions` (App Nap) fixes the latency of the death —from 11 s to
1 ms— but not the repaint, which stays erratic. It is in for the first reason.

## What travels

**What travels from the server to the Viewer is the SVG**, over an in-memory channel
— not the geometry and not a bitmap. mmdr's `Layout` **is** the PositionedScene, it
lives in the server thread and **does not cross**. The pixmap is a cache: `resvg`
rasterizes on demand in a worker thread and the event loop only uploads the texture.

## Measured numbers

macOS 26.6.2 arm64, `eframe` 0.36.1 / `winit` 0.30.13, debug build.

| | |
|---|---|
| Event loop up | 72–350 ms |
| stdio alive with the window open | answers in 0.1 ms |
| Repaint from the server thread, window in the background | 52–55 ms |
| Death on stdin closing, window in sight | exits on its own, code 0, 3.1 s |
| RSS of a session that never shows anything | **96.8 MB** |
| The same, after the first `show` | 97.2 MB |
| mmdr render | 3.3 ms (6 classes) – 14.9 ms (17) |

**99.6 % of the memory cost is paid on creating the event loop, not on showing the
window** — hence the deferred startup in ADR-0010.

## One process, one crate

The same reason settles the shape of the repo: **one binary crate with internal modules**,
plus the Launcher's shell script. Not a workspace and not a crate per layer — server and
Viewer share a process, so a crate boundary between them would be a border with nobody on
the other side. The package also carries a library target, which is how the integration
tests get in; what is delivered is still **one single file**, and no piece may become a
second versioned artifact that has to match versions with the first.

**How those modules are ordered among themselves: a module with a single consumer lives
inside it and is private; it rises to the root when a second one uses it.** So
`honest_limit` and `house_style` sit under `diagram`, `raster` under `viewer`, and
`flipchart` and `lifecycle` under `server` — while `mac` stays at the root, because the
Viewer and the startup both call into it, and one single border with the operating system
beats three. What the root shows is therefore the entry points and nothing else, and the
tree says out loud the same hierarchy the code already had.

The `wire` is the exception that proves it: four consumers, so the root is its place. Being
there is what lets `flipchart` hand the deck over without importing from `viewer`, and
leaves the Viewer a leaf of the graph — dumb by design, as the glossary claims and the
module tree now backs up.

## Considered options

- **Two processes with IPC** — rejected. A socket, an internal protocol, a handshake
  and orphaned processes, all to separate two halves that need nothing from that
  separation. Ephemerality would go back to being a rule somebody has to apply.
- **The event loop as the clock** — rejected by measurement. It is the obvious place
  for a timer, and it is exactly the thread macOS stops when the window is covered,
  which is the normal case.
- **Sending the geometry to the Viewer** — rejected. It would move the
  PositionedScene across a boundary it has no reason to cross, and put the drawing
  decision on the dumb side.
- **Sending the bitmap** — rejected. Zoom would look blurry, or it would ask for a
  render per mouse wheel notch.
