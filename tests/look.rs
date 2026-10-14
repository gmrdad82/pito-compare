use std::path::Path;

use pito_compare::{Chart, Kind};

fn pixels(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size().expect("a size")];
    let info = reader.next_frame(&mut buf).expect("a frame");
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

#[test]
fn the_readme_images_are_what_pcmp_draws_from_the_examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for data in [
        "examples/speed.toml",
        "examples/latency.json",
        "examples/build.toml",
        "examples/frames.toml",
        "bench/pcmp-vs-matplotlib.toml",
    ] {
        let path = root.join(data);
        let chart = Chart::load(&path).expect("example loads");
        let stem = path.file_stem().and_then(|s| s.to_str()).expect("stem");
        for (kind, canvas) in pito_compare::render(&chart, Kind::All).expect("renders") {
            let image = root.join("docs/images").join(format!("{stem}-{kind}.png"));
            let kept = std::fs::read(&image).expect("README image exists");
            let drawn = canvas.png().expect("encodes");
            assert!(
                pixels(&kept) == pixels(&drawn),
                "{} differs from what pcmp draws; run bin/images",
                image.display()
            );
        }
    }
}
