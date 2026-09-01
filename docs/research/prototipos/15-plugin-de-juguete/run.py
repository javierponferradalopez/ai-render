#!/usr/bin/env python3
"""Lanza un comando con timeout y deja los tiempos y la salida en ficheros."""
import subprocess, sys, time, os

timeout = float(sys.argv[1])
outbase = sys.argv[2]
cmd = sys.argv[3:]
t0 = time.time()
with open(outbase + ".out", "wb") as o, open(outbase + ".err", "wb") as e:
    p = subprocess.Popen(cmd, stdout=o, stderr=e, stdin=subprocess.DEVNULL)
    try:
        rc = p.wait(timeout)
        killed = False
    except subprocess.TimeoutExpired:
        p.kill(); p.wait(); rc = None; killed = True
t1 = time.time()
print("rc=%s killed=%s wall=%.2fs" % (rc, killed, t1 - t0))
