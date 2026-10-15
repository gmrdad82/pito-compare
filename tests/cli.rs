use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn pcmp(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .args(args)
        .env("NO_COLOR", "1")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("pcmp runs")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn a_bare_call_answers_like_clap() {
    let out = pcmp(&[]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(err.starts_with("error: "), "{err}");
    assert!(err.contains("\n\nUsage: pcmp "), "{err}");
    assert!(
        err.ends_with("\n\nFor more information, try '--help'.\n"),
        "{err}"
    );
}

#[test]
fn a_bad_data_file_is_one_line_and_exit_1() {
    let dir = scratch("bad");
    let data = dir.join("bad.json");
    std::fs::write(
        &data,
        r#"{"title":"X","better":"higher","parties":[{"name":"A"}],"rows":[{"label":"R","values":[1,2]}]}"#,
    )
    .expect("write");
    let huge = dir.join("huge.toml");
    std::fs::write(
        &huge,
        "title = \"X\"\nbetter = \"higher\"\nparties = [{ name = \"A\" }, { name = \"B\" }]\nrows = [{ label = \"R\", values = [1.7e308, 1.0], texts = [\"A\", \"B\"] }]\n",
    )
    .expect("write");
    for args in [
        vec![data.to_str().expect("utf-8")],
        vec![huge.to_str().expect("utf-8")],
        vec!["no-such-file.toml"],
    ] {
        let out = pcmp(&args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{err}");
        assert!(
            err.starts_with("pcmp: ") && err.lines().count() == 1,
            "{err}"
        );
    }
}

#[test]
fn each_chart_is_written_and_its_path_printed() {
    let dir = scratch("charts");
    let out = pcmp(&["examples/speed.toml", "--out", dir.to_str().expect("utf-8")]);
    assert_eq!(out.status.code(), Some(0));
    let printed: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(String::from)
        .collect();
    assert_eq!(printed.len(), 2);
    for (line, kind) in printed.iter().zip(["paired", "ratio"]) {
        assert!(line.ends_with(&format!("speed-{kind}.png")), "{line}");
        let bytes = std::fs::read(line).expect("png written");
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}

#[cfg(unix)]
#[test]
fn a_link_in_the_way_is_refused() {
    let dir = scratch("link");
    let target = dir.join("keep.txt");
    std::fs::write(&target, "keep").expect("write");
    std::os::unix::fs::symlink(&target, dir.join("speed-paired.png")).expect("link");
    let out = pcmp(&[
        "examples/speed.toml",
        "--kind",
        "paired",
        "--out",
        dir.to_str().expect("utf-8"),
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(std::fs::read_to_string(&target).expect("read"), "keep");
}
