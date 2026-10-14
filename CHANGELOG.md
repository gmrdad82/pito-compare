# Changelog

## v1.0.2 — 2026-10-14

### Added

- Each release carries pito-compare's documentation as docs-v1.0.2.tar.gz: an overview, install, eight guides with real commands and charts, the command line and data format reference, the library's API and this changelog.

## v1.0.1 — 2026-10-14

### Fixed

- A ratio chart's axis labels never draw over each other: when a wide spread puts the 2× SLOWER tick right beside 1× SAME, the 2× label gives way.

## v1.0.0 — 2026-10-14

pito-compare is 1.0: the charts, the data format and the library are the same as in 0.1.0.

### Changed

- The README shows every chart kind with an example, and pcmp's own numbers: 13 to 23 ms per chart from start to PNG, 39 to 64 times faster than matplotlib on the same data, a 1.2 MB binary and five direct dependencies.
- bench/speed.py measures pcmp against matplotlib, so anyone can run the comparison.
- The brand now reads PITO MD.

## v0.1.0 — 2026-10-11

### Added

- pcmp draws comparison charts as PNGs from a JSON or TOML file: paired bars for two or more parties, the balanced ratio chart and single bars, in the look of pito-codecs' benchmark charts.
- The pito_compare library draws the same charts from data built in code.
- Release binaries for Linux x86_64, Windows x86_64 and macOS on Apple Silicon, with SHA256SUMS.
