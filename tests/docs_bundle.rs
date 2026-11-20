#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, PoisonError};

static BUNDLE: Mutex<()> = Mutex::new(());

const POISONS: [&str; 2] = ["/home/nobody-9659/notes", "/Users/nobody-9659/notes"];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fresh(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("docs-bundle-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn tarball() -> String {
    format!("docs-v{}.tar.gz", env!("CARGO_PKG_VERSION"))
}

fn bundle(out: &Path, pages: Option<&Path>) -> Output {
    let mut dirs = Vec::new();
    if let Some(dir) = Path::new(env!("CARGO")).parent() {
        if !dir.as_os_str().is_empty() {
            dirs.push(dir.to_path_buf());
        }
    }
    dirs.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let mut command = Command::new("bash");
    command
        .arg(root().join("bin/docs-bundle"))
        .arg(out)
        .env("PATH", std::env::join_paths(dirs).expect("a PATH"));
    match pages {
        Some(pages) => command.env("DOCS_PAGES", pages),
        None => command.env_remove("DOCS_PAGES"),
    };
    command.output().expect("bash runs bin/docs-bundle")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a folder");
    for entry in std::fs::read_dir(from).expect("a folder").flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("a copied file");
        }
    }
}

fn files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("a folder").flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, found);
        } else {
            found.push(path);
        }
    }
}

fn holds(bytes: &[u8], needle: &str) -> bool {
    bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
}

#[test]
fn a_clean_bundle_holds_no_home_path_in_any_file() {
    let _lock = BUNDLE.lock().unwrap_or_else(PoisonError::into_inner);
    let base = fresh("clean");
    let out = base.join("out");
    let built = bundle(&out, None);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let archive = out.join(tarball());
    assert!(archive.is_file(), "no {}", archive.display());
    assert_eq!(
        String::from_utf8_lossy(&built.stdout).trim(),
        archive.display().to_string()
    );
    let site = base.join("site");
    std::fs::create_dir_all(&site).expect("scratch dir");
    let untarred = Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&site)
        .status()
        .expect("tar runs");
    assert!(untarred.success());
    let mut found = Vec::new();
    files(&site, &mut found);
    for generated in [
        "docs/reference/cli.md",
        "docs/changelog.md",
        "docs/skill.md",
        "docs/api/pito_compare/index.html",
    ] {
        assert!(
            site.join(generated).is_file(),
            "the bundle lacks {generated}"
        );
    }
    for file in found {
        let bytes = std::fs::read(&file).expect("a bundle file");
        for home in ["/home/", "/Users/"] {
            assert!(!holds(&bytes, home), "{} holds {home}", file.display());
        }
    }
}

#[test]
fn a_page_with_a_home_path_stops_the_bundle_and_leaves_no_tarball() {
    let _lock = BUNDLE.lock().unwrap_or_else(PoisonError::into_inner);
    for (n, poison) in POISONS.iter().enumerate() {
        let base = fresh(&format!("poisoned-{n}"));
        let pages = base.join("pages");
        copy(&root().join("docs/pages"), &pages);
        let page = pages.join("skill.md");
        let mut text = std::fs::read_to_string(&page).expect("a page");
        text.push_str(&format!("\nSee {poison}\n"));
        std::fs::write(&page, text).expect("a poisoned page");
        let out = base.join("out");
        let built = bundle(&out, Some(&pages));
        let said = String::from_utf8_lossy(&built.stderr);
        assert_eq!(built.status.code(), Some(1), "{poison}: {said}");
        assert!(
            said.contains("docs-bundle: a page carries a home path:"),
            "{poison}: {said}"
        );
        assert!(said.contains("skill.md"), "{poison}: {said}");
        let left: Vec<_> = std::fs::read_dir(&out).expect("the out folder").collect();
        assert!(left.is_empty(), "{poison}: the bundle left {left:?}");
        assert!(built.stdout.is_empty(), "{poison}: it printed a tarball");
    }
}
