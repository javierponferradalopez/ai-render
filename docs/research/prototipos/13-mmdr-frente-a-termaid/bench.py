"""Tiempos de mmdr, proceso completo incluido. Mediana de N ejecuciones."""
import subprocess, sys, time, statistics

MMDR = sys.argv[1] if len(sys.argv) > 1 else "mmdr"
CASOS = [
    ("cases/n06_mem.mmd", "classDiagram 6 nodos"),
    ("cases/n17_mem.mmd", "classDiagram 17 nodos"),
    ("arch.mmd", "flowchart 8 nodos + 4 grupos"),
]

for caso, label in CASOS:
    ts = []
    for _ in range(20):
        t = time.perf_counter()
        subprocess.run([MMDR, "-i", caso, "-o", "/dev/null", "-e", "svg"], capture_output=True)
        ts.append((time.perf_counter() - t) * 1000)
    print(f"{label:32} mediana {statistics.median(ts):6.1f} ms   min {min(ts):5.1f}   max {max(ts):6.1f}")
