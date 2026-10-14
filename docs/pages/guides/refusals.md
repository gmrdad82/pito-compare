---
title: Read a refusal
description: "What pcmp says when a data file or a command line is wrong, and how to fix each case."
sidebar:
  order: 6
---

pcmp never draws a wrong chart. When something in the data can't be drawn as written, it writes no PNG, prints one line on stderr and exits with 1. The line starts `pcmp: `, names the file, and points at the field, or at the line and column when the file doesn't parse.

## A value out of range

```
$ pcmp neg.toml
pcmp: neg.toml: groups[0].rows[0].values holds -612: values are finite and zero or more
```

Values are measurements, so zero or more. `groups[0].rows[0]` is the first row of the first group; a file with `rows` and no groups has them in one unnamed group, so the path reads the same.

## A character the font can't draw

```
$ pcmp char.toml
pcmp: char.toml: groups[0].rows[0].label: the chart font has no '€'; it draws A to Z, 0 to 9, space and . , : ; - + / ( ) % = < > ' _ | × * †
```

The chart font is a 5x7 pixel font with a fixed set of characters. Lower case is drawn in capitals, so it is never refused. Write `EUR` for `€`.

## An unknown key

```
$ pcmp key.toml
pcmp: key.toml: unknown field `color`, expected one of `title`, `unit`, `better`, `subtitle`, `date`, `parties`, `versus`, `groups`, `rows`, `notes`, `suffix`, `decimals`, `words` at line 3 column 1
```

A misspelt key would otherwise be dropped without a word, so it is refused. The party's colour is spelt `colour`, and it belongs to a party, not the chart.

## A row that doesn't match the parties

```
$ pcmp count.toml
pcmp: count.toml: groups[0].rows[0].values has 1 values for 2 parties
```

Every row has one value per party, in the parties' order.

## Text too long for its line

Titles, labels, party names and notes each have the room the 1080 px chart gives them, and pcmp refuses text that would run off it, naming the field. The title holds 41 characters; notes wrap at spaces, so only a single word longer than a line is refused. Shorten the text, or move detail into `notes`.

## A wrong command line

A command line pcmp can't parse is answered the way clap-based tools answer, with exit code 2:

```
$ pcmp speed.toml --kind nope
error: invalid value 'nope' for '--kind <KIND>'
  [possible values: all, paired, ratio, bars]

For more information, try '--help'.
```

`pcmp --help` prints every option; [the command line reference](../../reference/cli/) has them too.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Every chart was drawn. |
| `1` | The data couldn't be read or drawn, or a PNG couldn't be written. |
| `2` | The command line is wrong. |
