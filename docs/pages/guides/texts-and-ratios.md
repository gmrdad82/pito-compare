---
title: Show text or your own ratio
description: "Print your own text in place of a value, or give a row's ratio yourself when its values don't share a unit."
sidebar:
  order: 5
---

Sometimes the number on the bar isn't the one you want to show, or a row compares things that don't share a unit. A row's `texts` and `ratio` cover both.

## Text in place of a value

`texts` holds one string per party, printed beside the bars in place of the values. The bars are still drawn from `values`:

```toml
rows = [
  { label = "ENCODE SPEED", values = [240, 96], texts = ["240 FPS", "96 FPS"] },
]
```

The text is drawn in the chart's font, so it holds the same characters as every other field (see [the data format](../../reference/formats/#the-font)).

## Your own ratio

A row's `ratio` replaces the one its values give. Above 1 means the first party wins, below 1 that it loses, whichever way `better` points. That lets one ratio chart hold rows in different units: speed, size and start-up side by side.

```toml
title = "ENCODER AGAINST THE REFERENCE"
unit = "OURS AGAINST THE REFERENCE"
better = "higher"
parties = [{ name = "OURS" }, { name = "REFERENCE" }]
words = { better = "AHEAD", worse = "BEHIND", same = "LEVEL" }
rows = [
  { label = "ENCODE SPEED", values = [240, 96] },
  { label = "FILE SIZE", ratio = 0.89 },
  { label = "START-UP", ratio = 4.6 },
]
```

A row with a ratio and no values has nothing to draw as bars, so draw only the ratio chart:

```
$ pcmp encoder.toml --kind ratio
encoder-ratio.png
```

Asking for the paired bars of the same file is refused, and nothing is written:

```
$ pcmp encoder.toml
pcmp: encoder.toml: row 2 (FILE SIZE) has no values to draw as bars
```

A row needs values, a ratio or both; a ratio is a number above zero.

## A row the ratio chart can't draw

The ratio chart divides the two compared values, so both need to be above zero. The paired bars draw a zero as it is, but the ratio chart refuses the row unless it carries a `ratio`:

```
$ pcmp zero.toml --kind ratio
pcmp: zero.toml: row 1 (R) has no ratio: the compared values must be above zero
```
