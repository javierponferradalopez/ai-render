#!/usr/bin/env python3
"""Arranca Claude Code interactivo en un pty, espera, teclea comandos y guarda
la pantalla (sin secuencias ANSI) en <base>.screen."""
import os, pty, re, select, sys, time

base = sys.argv[1]; script = sys.argv[2]; cmd = sys.argv[3:]
# script: pares "espera:texto" separados por ';'  (texto vacio = solo esperar)
steps = [s for s in script.split(";") if s]
pid, fd = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.execvp(cmd[0], cmd)
buf = bytearray()
def drain(seconds):
    end = time.time() + seconds
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.3)
        if r:
            try: data = os.read(fd, 65536)
            except OSError: return False
            if not data: return False
            buf.extend(data)
    return True
for step in steps:
    wait, _, text = step.partition(":")
    if not drain(float(wait)): break
    if text:
        os.write(fd, (text + "\r").encode()); time.sleep(0.4)
drain(3)
os.close(fd)
try: os.waitpid(pid, os.WNOHANG)
except Exception: pass
raw = buf.decode("utf8", "replace")
clean = re.sub(r"\x1b\[[0-9;?]*[a-zA-Z]|\x1b\][^\x07]*\x07|\x1b[()][A-Z0-9]|\r", "", raw)
open(base + ".screen", "w").write(clean)
open(base + ".raw", "w", errors="replace").write(raw)
print("bytes=%d lineas=%d" % (len(raw), clean.count("\n")))
