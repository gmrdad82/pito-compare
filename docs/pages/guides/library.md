---
title: Draw from Rust
description: "Use the pito_compare library to draw the same charts from data built in code, with no command line and no clap."
sidebar:
  order: 7
---

The charts pcmp draws come from the `pito_compare` library, and a Rust program can call it directly: build the data in code, or read the same JSON and TOML files, and get PNGs back.

## Add it

```toml
[dependencies]
pito-compare = { git = "https://github.com/gmrdad82/pito-compare", tag = "vX.Y.Z", default-features = false }
```

Replace `vX.Y.Z` with a [tag](https://github.com/gmrdad82/pito-compare/tags). `default-features = false` leaves out the command line and its clap; the library then needs png, serde, serde_json and toml.

## Build a chart in code

```rust
use pito_compare::{Better, Chart, Group, Kind, Row};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = Chart::new(
        "DECODE SPEED",
        "FRAMES PER SECOND",
        Better::Higher,
        &["OURS", "THEIRS"],
    );
    chart.suffix = " FPS".into();
    chart.groups.push(Group {
        name: "CPU".into(),
        rows: vec![
            Row::new("H.264 1080P", &[412.0, 298.0]),
            Row::new("H.265 1080P", &[251.0, 187.0]),
        ],
    });
    for (kind, canvas) in pito_compare::render(&chart, Kind::All)? {
        canvas.save(format!("decode-{kind}.png").as_ref())?;
    }
    Ok(())
}
```

This writes `decode-paired.png` and `decode-ratio.png`. `render` checks the chart first, with the same one-line errors pcmp prints, and returns each chart's kind with its canvas: `paired`, `ratio`, `bars`, or `bars-<party>` for each party.

## Read a data file

`Chart::load` reads a `.json` or `.toml` file in [the data format](../../reference/formats/); `Chart::parse` reads text you already hold:

```rust
use pito_compare::{Chart, Format};

let chart = Chart::load("speed.toml".as_ref())?;
pito_compare::paired(&chart)?.save("speed-paired.png".as_ref())?;

let text = r#"{"title": "PARSE SPEED", "better": "higher",
    "parties": [{"name": "V2"}, {"name": "V1"}],
    "rows": [{"label": "CONFIG FILES", "values": [612, 240]}]}"#;
let chart = Chart::parse(text, Format::Json)?;
let png: Vec<u8> = pito_compare::ratios(&chart)?.png()?;
```

## One chart at a time

| Function | Draws |
|---|---|
| `paired(&chart)` | the paired bars |
| `ratios(&chart)` | the balanced ratio chart |
| `bars(&chart, party)` | one party's single bars |
| `render(&chart, kind)` | what `pcmp --kind` draws, named |
| `render_each(&chart, kind, f)` | the same, handing each chart to `f(name, canvas)` as soon as it is drawn, so only one is held at a time |

Each returns a `Canvas`. `png()` encodes it to bytes, `write_png` writes it to any `Write`, and `save` writes a file. Errors are `pito_compare::Error`, whose message is the line pcmp would print.

Every public type and function is in [the API reference](../../api/pito_compare/index.html).
