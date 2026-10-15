# Changelog

## v1.1.0 — 2026-10-15

### Added

- The library's `render_each` hands each chart over as soon as it is drawn, so only one is held in memory at a time; pcmp now saves each chart before drawing the next.
- The README's Licence section shows the PITO MD wordmark.

### Changed

- Charts are about 30% faster from start to PNG on disk (paired bars 9.3 to 6.6 ms): the PNGs are compressed at the default level, with the same pixels and about 1% more bytes.
- A data file is refused in one line when a value is above 1e15, a ratio is outside 1e-15 to 1e15, or it names more than 12 parties.

### Fixed

- A huge value with `texts` no longer makes pcmp hang and fill memory.
- pcmp refuses to write a chart through a symlink at its output path, so it never overwrites the file the link points to.

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
