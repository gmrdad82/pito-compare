---
title: Measure pcmp against matplotlib
description: "Run the benchmark that times pcmp against matplotlib drawing the same data, and read its numbers."
sidebar:
  order: 8
---

pcmp draws its own benchmark. `bench/speed.py` in the repository times pcmp drawing each chart kind, then matplotlib drawing the same data as a similar dark 1080 px chart, and prints the numbers.

## Run it

On Linux or macOS, you need a clone of the repository, Rust 1.97 or newer, and Python with matplotlib installed:

```
git clone https://github.com/gmrdad82/pito-compare
cd pito-compare
cargo build --release
python3 bench/speed.py
```

For each chart kind it starts a new process per chart, 50 times, and takes the median, for pcmp and for matplotlib alike. Nothing but the benchmark needs Python.

## What it measured

On 2026-10-15, on an AMD Ryzen 9 9950X (32 threads) under Linux, with Python 3.14 and matplotlib 3.11, on a lightly loaded machine (load 2 on 32 threads). `bench/pcmp-vs-matplotlib.toml` holds the numbers it printed, and pcmp drew them:

![Paired bars, lower is better: start to PNG on disk, pcmp 6.4, 4.9 and 5.2 ms against matplotlib 408, 387 and 380 ms; 64×, 79× and 73× faster](../images/pcmp-vs-matplotlib-paired.png)

![The balanced ratio chart of the same numbers: pcmp 64×, 79× and 73× faster than matplotlib](../images/pcmp-vs-matplotlib-ratio.png)

| | pcmp | matplotlib |
|---|---|---|
| Paired bars | 6.4 ms | 408 ms |
| Balanced ratios | 4.9 ms | 387 ms |
| Single bars | 5.2 ms | 380 ms |
| Peak memory | 13.4 MB | 82.6 MB |

## Reading the numbers

Most of matplotlib's time is Python and its import, but not all: in one run, drawing and saving after the import took 126 to 151 ms per chart, over twenty times pcmp's whole run. The matplotlib chart is the plainer of the two (no tags, no pixel font), which favours it.

## Chart your own benchmark

The bench's numbers are an ordinary data file. To chart your own, write your timings the same way, with `better = "lower"` for times, and draw them:

```
$ pcmp pcmp-vs-matplotlib.toml
pcmp-vs-matplotlib-paired.png
pcmp-vs-matplotlib-ratio.png
```
