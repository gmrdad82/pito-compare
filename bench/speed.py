import os
import platform
import resource
import statistics
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PCMP = os.environ.get("PCMP", os.path.join(ROOT, "target", "release", "pcmp"))
OUT = os.path.join(ROOT, "tmp", "bench")
RUNS = int(os.environ.get("RUNS", "50"))
CASES = [("examples/speed.toml", "paired"), ("examples/speed.toml", "ratio"), ("examples/build.toml", "bars")]


def median_ms(command):
    times = []
    for _ in range(RUNS):
        start = time.perf_counter()
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL, cwd=ROOT)
        times.append((time.perf_counter() - start) * 1000)
    return statistics.median(times)


os.makedirs(OUT, exist_ok=True)
print(f"{platform.processor() or platform.machine()}, {os.cpu_count()} threads, load {os.getloadavg()[0]:.1f}, {RUNS} runs each, median")
for data, kind in CASES:
    ms = median_ms([PCMP, data, "--kind", kind, "--out", OUT])
    print(f"pcmp {kind}: {ms:.1f} ms")
print(f"pcmp peak memory: {resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024:.1f} MB")
script = os.path.join(ROOT, "bench", "matplotlib_chart.py")
for data, kind in CASES:
    png = os.path.join(OUT, f"matplotlib-{kind}.png")
    ms = median_ms([sys.executable, "-I", script, data, kind, png])
    split = subprocess.run([sys.executable, "-I", script, data, kind, png], check=True, capture_output=True, text=True, cwd=ROOT).stdout.split()
    print(f"matplotlib {kind}: {ms:.1f} ms (one run: import {split[0]} ms, draw and save {split[1]} ms)")
print(f"matplotlib peak memory: {resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024:.1f} MB")
