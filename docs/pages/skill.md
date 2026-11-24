---
title: The Claude skill
description: "pcmp comes with a Claude skill: what it teaches an AI coding agent, where it lands, and how installing and updating treat it."
sidebar:
  order: 3
---

pcmp comes with a Claude skill. It teaches your AI coding agent to write the data file, pick the chart, style it and read a refusal, so you say "chart these two benchmark runs" and get the PNGs.

## What it teaches

A `SKILL.md` with a description and triggers, the safety rules, how to find a command through `pcmp --help`, a framework for picking the chart, example requests and what is out of scope, and four topic guides:

| Guide | What it covers |
|---|---|
| `data-file.md` | The JSON and TOML data file: every key, the limits and the font's characters. |
| `charts.md` | Paired bars, the balanced ratio chart and single bars: which to draw and how a ratio reads. |
| `styling.md` | Party colours, the words of the tags, decimals, suffix, marks, notes, texts and your own ratio. |
| `output.md` | `--out`, `--kind`, the PNG names, stdout, the exit codes and how to read a refusal. |

The skill is built into the `pcmp` binary, so the copy always matches the pcmp that installed it.

## Where it lands

`pcmp skill install` writes the skill as files into:

| Agent | Folder |
|---|---|
| Claude Code | `~/.claude/skills/pcmp` |
| Other agents | `~/.agents/skills/pcmp` |
| Codex | `~/.codex/skills/pcmp`, when `~/.codex` exists |
| Pi | `~/.pi/agent/skills/pcmp`, when `~/.pi` exists |

On Windows these are under your user folder (`%USERPROFILE%`). An older link to a store folder from earlier pcmp versions is replaced by real files.

## Installing and updating

The [install lines](../install/) for Linux, macOS and Windows run `pcmp skill install` after unpacking pcmp, and `bin/install` from a clone does the same. Running them again is the update: the binary and the skill are replaced together, an older copy of the skill is replaced, and a file a new version drops is removed.

`pcmp skill install` alone writes the skill again, and `pcmp skill status` says, for each folder, whether it is installed and matches this pcmp:

```
$ pcmp skill status
Claude skill: installed at ~/.claude/skills/pcmp, matches pcmp 1.5.0
Agents skill: installed at ~/.agents/skills/pcmp, matches pcmp 1.5.0
```

A folder that is not installed reads `not installed`, and one whose files differ from this pcmp reads `does not match pcmp 1.5.0; pcmp skill install refreshes it`.
