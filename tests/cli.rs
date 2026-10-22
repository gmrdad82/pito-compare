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

fn printed(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(String::from)
        .collect()
}

fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn each_kind_writes_only_its_files_and_prints_their_paths() {
    for (data, kind, want) in [
        ("examples/speed.toml", "paired", vec!["speed-paired.png"]),
        ("examples/speed.toml", "ratio", vec!["speed-ratio.png"]),
        (
            "examples/latency.json",
            "bars",
            vec![
                "latency-bars-cache-v1.png",
                "latency-bars-cache-v2.png",
                "latency-bars-no-cache.png",
            ],
        ),
        ("examples/build.toml", "all", vec!["build-bars.png"]),
    ] {
        let dir = scratch(&format!("kind-{kind}"));
        let out = pcmp(&[data, "--kind", kind, "--out", dir.to_str().expect("utf-8")]);
        assert_eq!(out.status.code(), Some(0), "{kind}");
        assert!(out.stderr.is_empty(), "{kind}");
        assert_eq!(files(&dir), want, "{kind}");
        let mut shown = printed(&out);
        shown.sort();
        let expected: Vec<String> = want
            .iter()
            .map(|f| dir.join(f).display().to_string())
            .collect();
        assert_eq!(shown, expected, "{kind}");
        for p in &shown {
            let bytes = std::fs::read(p).expect("png written");
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }
}

#[test]
fn a_missing_out_folder_is_made() {
    let dir = scratch("made").join("new/deeper");
    let out = pcmp(&["examples/build.toml", "--out", dir.to_str().expect("utf-8")]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(files(&dir), ["build-bars.png"]);
}

#[test]
fn an_out_folder_that_cannot_be_made_is_one_line_and_exit_1() {
    let dir = scratch("unmade");
    let file = dir.join("file");
    std::fs::write(&file, "x").expect("write");
    let out = pcmp(&[
        "examples/speed.toml",
        "--out",
        file.join("charts").to_str().expect("utf-8"),
    ]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(out.stdout.is_empty());
    assert!(
        err.starts_with("pcmp: cannot create ") && err.lines().count() == 1,
        "{err}"
    );
}

#[test]
fn the_default_out_is_the_working_folder_and_prints_bare_names() {
    let dir = scratch("cwd");
    std::fs::copy("examples/build.toml", dir.join("build.toml")).expect("copy");
    let out = Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .arg("build.toml")
        .current_dir(&dir)
        .output()
        .expect("pcmp runs");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(printed(&out), ["build-bars.png"]);
    assert!(dir.join("build-bars.png").is_file());
}

#[cfg(unix)]
#[test]
fn a_data_file_whose_stem_is_not_utf8_draws_as_chart() {
    use std::os::unix::ffi::OsStrExt;
    let dir = scratch("stem");
    let data = dir.join(std::ffi::OsStr::from_bytes(b"\xff\xfe.toml"));
    if let Err(e) = std::fs::copy("examples/build.toml", &data) {
        eprintln!("skipped: this file system refuses a non-UTF-8 name ({e})");
        return;
    }
    let out_dir = dir.join("out");
    let out = Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .arg(&data)
        .arg("--out")
        .arg(&out_dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("pcmp runs");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(files(&out_dir), ["chart-bars.png"]);
    let shown = printed(&out);
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(shown[0].ends_with("chart-bars.png"), "{shown:?}");
}

#[test]
fn help_goes_to_stdout_and_exits_0() {
    for flag in ["--help", "-h"] {
        let out = pcmp(&[flag]);
        let text = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(0), "{flag}");
        assert!(out.stderr.is_empty(), "{flag}");
        assert!(text.contains("Usage: pcmp"), "{text}");
        assert!(text.contains("--kind"), "{text}");
    }
}

#[test]
fn an_unknown_kind_answers_like_clap() {
    let out = pcmp(&["examples/speed.toml", "--kind", "pie"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(out.stdout.is_empty());
    assert!(
        err.starts_with("error: invalid value 'pie' for '--kind"),
        "{err}"
    );
    assert!(
        err.ends_with("\n\nFor more information, try '--help'.\n"),
        "{err}"
    );
}

#[test]
fn bars_for_parties_whose_names_collide_print_distinct_paths() {
    let dir = scratch("collide");
    let data = dir.join("clash.toml");
    std::fs::write(
        &data,
        "title = \"X\"\nbetter = \"higher\"\nparties = [{ name = \"2\" }, { name = \"%%\" }, { name = \"3\" }, { name = \"3.\" }]\nrows = [{ label = \"R\", values = [1.0, 2.0, 3.0, 4.0] }]\n",
    )
    .expect("write");
    let out_dir = dir.join("out");
    let out = pcmp(&[
        data.to_str().expect("utf-8"),
        "--kind",
        "bars",
        "--out",
        out_dir.to_str().expect("utf-8"),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut shown = printed(&out);
    assert_eq!(shown.len(), 4, "{shown:?}");
    shown.sort();
    shown.dedup();
    assert_eq!(shown.len(), 4, "{shown:?}");
    assert_eq!(
        files(&out_dir),
        [
            "clash-bars-2-2.png",
            "clash-bars-2.png",
            "clash-bars-3.png",
            "clash-bars-4.png",
        ]
    );
}

#[test]
fn a_chart_that_loads_but_cannot_be_drawn_names_the_data_file() {
    let dir = scratch("tall");
    let data = dir.join("tall.toml");
    let rows: Vec<String> = (0..300)
        .map(|i| format!("{{ label = \"R{i}\", values = [1.0, 2.0] }}"))
        .collect();
    std::fs::write(
        &data,
        format!(
            "title = \"X\"\nbetter = \"higher\"\nparties = [{{ name = \"A\" }}, {{ name = \"B\" }}]\nrows = [{}]\n",
            rows.join(", ")
        ),
    )
    .expect("write");
    let out_dir = dir.join("out");
    let out = pcmp(&[
        data.to_str().expect("utf-8"),
        "--kind",
        "paired",
        "--out",
        out_dir.to_str().expect("utf-8"),
    ]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(out.stdout.is_empty());
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with(&format!("pcmp: {}: ", data.display())),
        "{err}"
    );
    assert!(err.contains("px tall"), "{err}");
    assert!(!out_dir.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn a_full_stdout_still_writes_every_chart() {
    let dir = scratch("full");
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full");
    let status = Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .args(["examples/speed.toml", "--out"])
        .arg(&dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdout(full)
        .status()
        .expect("pcmp runs");
    assert_eq!(status.code(), Some(0));
    assert_eq!(files(&dir), ["speed-paired.png", "speed-ratio.png"]);
}
