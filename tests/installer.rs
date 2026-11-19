#![cfg(unix)]

use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FILES: [&str; 5] = [
    "SKILL.md",
    "data-file.md",
    "charts.md",
    "styling.md",
    "output.md",
];

const OLD_PCMP: &str = "#!/bin/sh\necho 'pcmp 1.2.0'\n";

fn fresh(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("installer-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn install(home: &Path) -> Output {
    Command::new("bash")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("bin/install"))
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("PCMP_BINARY", env!("CARGO_BIN_EXE_pcmp"))
        .output()
        .expect("bash runs bin/install")
}

fn names(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .expect("a folder")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

fn skill(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/pcmp")
            .join(name),
    )
    .expect("a skill file")
}

fn wanted() -> Vec<String> {
    let mut all: Vec<String> = FILES.iter().map(|s| (*s).to_owned()).collect();
    all.push(".pcmp-skill".to_owned());
    all.sort();
    all
}

fn works(out: &Output) {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn lands_together(home: &Path) {
    let opt = home.join(".local/opt/pcmp");
    let installed = opt.join("pcmp");
    assert_eq!(
        std::fs::read(&installed).expect("the binary"),
        std::fs::read(env!("CARGO_BIN_EXE_pcmp")).expect("the built binary")
    );
    assert_ne!(
        std::fs::metadata(&installed)
            .expect("binary")
            .permissions()
            .mode()
            & 0o111,
        0
    );
    assert_eq!(
        std::fs::read_link(home.join(".local/bin/pcmp")).expect("a link"),
        installed
    );
    let store = opt.join("skills/pcmp");
    assert_eq!(names(&store), wanted());
    for name in FILES {
        assert_eq!(
            std::fs::read_to_string(store.join(name)).expect("skill"),
            skill(name),
            "{name}"
        );
    }
    for rel in [".claude/skills/pcmp", ".agents/skills/pcmp"] {
        assert_eq!(
            std::fs::read_link(home.join(rel)).expect("a link"),
            store,
            "{rel}"
        );
    }
    let status = Command::new(&installed)
        .args(["skill", "status"])
        .env_clear()
        .env("HOME", home)
        .output()
        .expect("installed pcmp runs");
    let text = String::from_utf8_lossy(&status.stdout);
    assert_eq!(text.matches(", matches pcmp ").count(), 2, "{text}");
}

#[test]
fn the_installer_lands_the_binary_and_the_skill_together_in_a_temporary_home() {
    let home = fresh("fresh");
    let out = install(&home);
    works(&out);
    lands_together(&home);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Claude skill: installed at "), "{text}");
    assert!(
        text.trim_end()
            .ends_with(&format!("pcmp {}", env!("CARGO_PKG_VERSION"))),
        "{text}"
    );
}

#[test]
fn an_update_from_an_older_version_replaces_the_binary_and_the_skill() {
    let home = fresh("update");
    let opt = home.join(".local/opt/pcmp");
    let store = opt.join("skills/pcmp");
    std::fs::create_dir_all(&store).expect("store");
    std::fs::write(opt.join("pcmp"), OLD_PCMP).expect("old binary");
    std::fs::write(store.join("SKILL.md"), "the 1.2.0 skill\n").expect("old skill");
    std::fs::write(store.join("dropped.md"), "no longer shipped\n").expect("old guide");
    std::fs::write(store.join(".pcmp-skill"), "1.2.0\n").expect("old mark");
    let bin = home.join(".local/bin");
    std::fs::create_dir_all(&bin).expect("bin");
    std::fs::write(bin.join("pcmp"), OLD_PCMP).expect("old plain binary");
    std::fs::create_dir_all(home.join(".claude/skills")).expect("claude");
    symlink(&store, home.join(".claude/skills/pcmp")).expect("old link");
    let copy = home.join(".agents/skills/pcmp");
    std::fs::create_dir_all(&copy).expect("copy");
    std::fs::write(copy.join("SKILL.md"), "a copied 1.2.0 skill\n").expect("copy");
    std::fs::write(copy.join(".pcmp-skill"), "1.2.0\n").expect("copy mark");
    let out = install(&home);
    works(&out);
    lands_together(&home);
    assert!(!store.join("dropped.md").exists());
}

#[test]
fn installing_again_changes_nothing_and_leaves_a_foreign_skill_folder_alone() {
    let home = fresh("again");
    works(&install(&home));
    works(&install(&home));
    lands_together(&home);
    let theirs = home.join(".claude/skills/pcmp");
    std::fs::remove_file(&theirs).expect("unlink");
    std::fs::create_dir(&theirs).expect("theirs");
    std::fs::write(theirs.join("SKILL.md"), "somebody else's\n").expect("theirs");
    let out = install(&home);
    works(&out);
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("left alone"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert_eq!(
        std::fs::read_to_string(theirs.join("SKILL.md")).expect("kept"),
        "somebody else's\n"
    );
    assert_eq!(
        std::fs::read_link(home.join(".agents/skills/pcmp")).expect("a link"),
        home.join(".local/opt/pcmp/skills/pcmp")
    );
}

#[test]
fn the_installer_refuses_arguments() {
    let out = Command::new("bash")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("bin/install"))
        .arg("--uninstall")
        .env("HOME", fresh("args"))
        .output()
        .expect("bash runs");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr), "usage: bin/install\n");
}
