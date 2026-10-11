# Inspiration

pito-compare is the benchmark chart drawer of [pito-codecs](https://github.com/gmrdad82/pito-codecs), made into a library and a command anyone can use.

The source is `examples/bench/charts.rs` and its callers in `examples/bench.rs`, at commit [`312dca20573b3700cd483e2a1b7a6408331803b1`](https://github.com/gmrdad82/pito-codecs/tree/312dca20573b3700cd483e2a1b7a6408331803b1/examples/bench) of pito-codecs, © Catalin Ilinca, MIT, the same owner and licence as this repository. It was copied in, not linked: nothing here reads pito-codecs at build or run time.

What came over unchanged:

- The look: background `#101319`, text `#E8EBF0`, dim `#8A92A0`, grid `#2C323E`, separator `#464E5C`, ours `#DC6E22`, theirs `#4A8EE6`, faster `#4AC97D` and slower `#EB5858`.
- The hand-drawn 5x7 pixel font, scaled up and drawn in capitals, and every glyph of it.
- The sizes: 1080 px wide, a 40 px margin, 124 px per paired row, 66 px per ratio row, 50 px for a group's heading, 30 px per note line.
- The paired chart (one linear axis from zero, the value beside each bar, "n× faster" or "n× slower" per row) and the balanced ratio chart (1× in the middle, a symmetric log range from 2× to 1000×, ticks that make room for each other).
- The canvas and its PNG writer, through the `png` crate; no plotting library.

What is new: the paired chart takes any number of parties, a third kind draws single bars, the words and the colours can be chosen, and the data comes from a JSON or TOML file through the `pcmp` command. With two parties and the source's names, pcmp's paired and ratio charts were checked pixel for pixel against the source's own functions on the examples in `examples/`, and matched.
