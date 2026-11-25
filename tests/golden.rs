use std::path::Path;

use pito_compare::{Chart, Kind, render_at};

fn pixels(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let picture = pito_codecs::png::Decoder::new()
        .decode_picture(bytes, pito_codecs::png::Transform::Identity)
        .expect("a PNG");
    (picture.width, picture.height, picture.data)
}

const GOLDENS: [(&str, &str, i64, Kind); 17] = [
    ("405-paired", "examples/speed.toml", 405, Kind::Paired),
    ("405-ratio", "examples/speed.toml", 405, Kind::Ratio),
    ("405-bars", "examples/build.toml", 405, Kind::Bars),
    (
        "405-bare-paired",
        "tests/golden/alert-two.toml",
        405,
        Kind::Paired,
    ),
    (
        "405-bare-ratio",
        "tests/golden/alert-two.toml",
        405,
        Kind::Ratio,
    ),
    (
        "405-bare-bars",
        "tests/golden/alert-one.toml",
        405,
        Kind::Bars,
    ),
    ("540-paired", "examples/speed.toml", 540, Kind::Paired),
    ("540-ratio", "examples/speed.toml", 540, Kind::Ratio),
    ("540-bars", "examples/build.toml", 540, Kind::Bars),
    (
        "540-bare-paired",
        "tests/golden/alert-two.toml",
        540,
        Kind::Paired,
    ),
    (
        "540-bare-ratio",
        "tests/golden/alert-two.toml",
        540,
        Kind::Ratio,
    ),
    (
        "540-bare-bars",
        "tests/golden/alert-one.toml",
        540,
        Kind::Bars,
    ),
    (
        "405-max-high-bars",
        "tests/golden/max-high.toml",
        405,
        Kind::Bars,
    ),
    (
        "405-max-low-bars",
        "tests/golden/max-low.toml",
        405,
        Kind::Bars,
    ),
    (
        "1080-bare-paired",
        "tests/golden/alert-two.toml",
        1080,
        Kind::Paired,
    ),
    (
        "1080-bare-ratio",
        "tests/golden/alert-two.toml",
        1080,
        Kind::Ratio,
    ),
    (
        "1080-bare-bars",
        "tests/golden/alert-one.toml",
        1080,
        Kind::Bars,
    ),
];

#[test]
fn the_golden_charts_are_what_pcmp_draws() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let update = std::env::var_os("PCMP_UPDATE_GOLDEN").is_some();
    for (name, data, width, kind) in GOLDENS {
        let chart = Chart::load(&root.join(data)).expect("data loads");
        let drawn = render_at(&chart, kind, width)
            .expect("renders")
            .remove(0)
            .1
            .png()
            .expect("encodes");
        let path = root.join("tests/golden").join(format!("{name}.png"));
        if update {
            std::fs::write(&path, &drawn).expect("golden written");
            continue;
        }
        let kept = std::fs::read(&path).expect("golden exists");
        assert!(
            pixels(&kept) == pixels(&drawn),
            "{} differs from what pcmp draws; run the tests with PCMP_UPDATE_GOLDEN=1 to redraw it",
            path.display()
        );
    }
}
