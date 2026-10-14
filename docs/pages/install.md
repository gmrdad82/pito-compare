---
title: Install
description: "Install pcmp on Linux, macOS or Windows, or build it from source, then draw a first chart."
sidebar:
  order: 2
---

pcmp runs on Linux (x86_64), Windows (x86_64) and macOS (Apple Silicon). Each [release](https://github.com/gmrdad82/pito-compare/releases/latest) carries a binary for all three, with a `SHA256SUMS` file. pcmp is one file and needs nothing else to run.

## Linux

```
v=$(basename "$(curl -fsSLo /dev/null -w '%{url_effective}' https://github.com/gmrdad82/pito-compare/releases/latest)")
mkdir -p ~/.local/bin && curl -fsSL "https://github.com/gmrdad82/pito-compare/releases/download/$v/pcmp-$v-linux-x86_64.tar.gz" | tar -xz -C ~/.local/bin pcmp
```

## macOS

```
v=$(basename "$(curl -fsSLo /dev/null -w '%{url_effective}' https://github.com/gmrdad82/pito-compare/releases/latest)")
mkdir -p ~/.local/bin && curl -fsSL "https://github.com/gmrdad82/pito-compare/releases/download/$v/pcmp-$v-macos-arm64.tar.gz" | tar -xz -C ~/.local/bin pcmp
```

On Linux and macOS, `~/.local/bin` needs to be on your `PATH`.

## Windows

In PowerShell:

```
$v = (Invoke-RestMethod https://api.github.com/repos/gmrdad82/pito-compare/releases/latest).tag_name
Invoke-WebRequest "https://github.com/gmrdad82/pito-compare/releases/download/$v/pcmp-$v-windows-x86_64.zip" -OutFile "$env:TEMP\pcmp.zip"
Expand-Archive "$env:TEMP\pcmp.zip" "$env:LOCALAPPDATA\pcmp" -Force
[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ";$env:LOCALAPPDATA\pcmp", 'User')
```

Open a new terminal afterwards, so it sees the new `PATH`.

## From source

On any of the three, with Rust 1.97 or newer:

```
cargo install --locked --git https://github.com/gmrdad82/pito-compare --tag vX.Y.Z pito-compare
```

Replace `vX.Y.Z` with a [tag](https://github.com/gmrdad82/pito-compare/tags).

## Check the download

Each release's `SHA256SUMS` lists every file attached to it. On Linux, next to the downloaded archive:

```
sha256sum --check --ignore-missing SHA256SUMS
```

## Quick start

Save this as `demo.toml`:

```toml
title = "PARSE SPEED"
unit = "MEGABYTES PER SECOND"
better = "higher"
suffix = " MB/S"
parties = [{ name = "PARSER V2" }, { name = "PARSER V1" }]
rows = [
  { label = "CONFIG FILES", values = [612, 240] },
  { label = "LOG LINES", values = [455, 420] },
  { label = "DATA DUMPS", values = [1210, 388] },
]
```

then draw it:

```
$ pcmp demo.toml --out charts
charts/demo-paired.png
charts/demo-ratio.png
```

`charts/` is made if it is missing. Two parties draw two charts: the paired bars and the balanced ratios. [Compare two versions](../guides/compare-two/) explains both, and [the data format](../reference/formats/) has every field.
