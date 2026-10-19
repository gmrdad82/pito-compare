---
title: pito-compare
description: "pcmp draws benchmark comparison charts as PNGs from a small JSON or TOML file: paired bars, balanced ratios and single bars."
sidebar:
  order: 1
---

![A terminal: cat demo.toml shows a small TOML file with a title, a unit, higher is better, two parties and three rows; pcmp demo.toml --out charts prints charts/demo-paired.png and charts/demo-ratio.png; pcmp demo.toml --kind bars --out charts prints one PNG per party; file shows a 1080 px wide PNG.](images/draw.gif)

pito-compare draws benchmark comparison charts as PNGs, part of [PITO MD](https://pitomd.com). You write the numbers in a small JSON or TOML file, and `pcmp` draws them as dark, crisp charts in a hand-drawn pixel font: two or more parties side by side on one axis, how many times faster or slower one is than another, or plain bars. A chart takes 5 to 7 ms from start to PNG on disk, 63 to 77 times faster than matplotlib drawing the same data.

![Paired bars: parse speed of parser v2 against parser v1 in two groups, with 2.5× faster, same speed and 3.2× slower tags, a marked row and two notes](images/speed-paired.png)

## At a glance

| | pcmp |
|---|---|
| Time per chart, start to PNG on disk | **5.7 ms** single bars, **5.5 ms** balanced ratios, **7.1 ms** paired bars |
| Against matplotlib 3.11, same data | **63× to 77× faster** (408 to 447 ms per chart) |
| Peak memory | **12.9 MB** (matplotlib: 82.5 MB) |
| Binary | **1.2 MB** (Linux x86_64, release profile, stripped) |
| Dependencies | **5** direct: png, serde, serde_json, toml, and clap for the command line |
| Needs to run | nothing: one binary, no Python, no plotting library, no fonts on the system |

Measured on 2026-10-14 on an AMD Ryzen 9 9950X (32 threads) under Linux while the machine was busy with other work; times are the median of 50 runs, a new process per chart. [Measure it yourself](guides/benchmark/).

## What it is

Benchmark results read best as a picture, and a picture that looks the same every time. pito-compare is the chart drawer of [pito-codecs](https://docs.pitomd.com/pito-codecs/)' benchmarks made into a tool, with its exact look: the colours, the 5x7 pixel font, 1080 px wide. There is no plotting library: the canvas, the font and the PNG writer are a few hundred lines of Rust, so a chart is drawn the same, pixel for pixel, on every machine.

It comes two ways: `pcmp`, the command line, and `pito_compare`, a Rust library that draws the same charts from data built in code.

## Where to go next

- [Install pcmp](install/) and draw a first chart.
- [Compare two versions](guides/compare-two/): paired bars and the balanced ratio chart.
- [Compare three or more](guides/three-or-more/): pick who the tags compare with, and charts where lower is better.
- [Draw single bars](guides/single-bars/) when there is nothing to compare.
- [Pick the colours](guides/colours/) of each party.
- [Show text or your own ratio](guides/texts-and-ratios/) in place of a value.
- [Read a refusal](guides/refusals/): what pcmp says when a data file is wrong.
- [Draw from Rust](guides/library/) with the library.
- [See pcmp against matplotlib](guides/benchmark/).
- Reference: [the command line](reference/cli/), [the data format](reference/formats/), [the API](api/pito_compare/index.html), [the changelog](changelog/).
