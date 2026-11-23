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
    for rel in [".claude/skills/pcmp", ".agents/skills/pcmp"] {
        let dir = home.join(rel);
        assert!(
            !std::fs::symlink_metadata(&dir)
                .expect("there")
                .file_type()
                .is_symlink()
        );
        assert_eq!(names(&dir), wanted(), "{rel}");
        for name in FILES {
            assert_eq!(
                std::fs::read_to_string(dir.join(name)).expect("skill"),
                skill(name),
                "{name}"
            );
        }
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
    assert!(!home.join(".agents/skills/pcmp/dropped.md").exists());
    assert_eq!(
        std::fs::read_to_string(store.join("SKILL.md")).expect("store"),
        "the 1.2.0 skill\n"
    );
}

#[test]
fn installing_again_changes_nothing() {
    let home = fresh("again");
    works(&install(&home));
    works(&install(&home));
    lands_together(&home);
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

fn release_install(name: &str, machine: &str) -> (Output, String) {
    let home = fresh(name);
    let bins = home.join("fakebin");
    std::fs::create_dir_all(&bins).expect("fake bin");
    let uname = bins.join("uname");
    std::fs::write(&uname, format!("#!/bin/sh\necho {machine}\n")).expect("uname");
    std::fs::set_permissions(&uname, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    let stage = home.join("stage");
    let tag = home.join("stage/v9.9.9");
    std::fs::create_dir_all(&tag).expect("stage");
    let pack = |asset: &str| {
        let status = Command::new("tar")
            .arg("-czf")
            .arg(tag.join(asset))
            .arg("-C")
            .arg(Path::new(env!("CARGO_BIN_EXE_pcmp")).parent().unwrap())
            .arg("pcmp")
            .status()
            .expect("tar");
        assert!(status.success());
    };
    pack("pcmp-v9.9.9-linux-x86_64.tar.gz");
    pack("pcmp-v9.9.9-linux-aarch64.tar.gz");
    let path = format!(
        "{}:{}",
        bins.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("bash")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("bin/install"))
        .env_clear()
        .env("PATH", path)
        .env("HOME", &home)
        .env("PCMP_RELEASE", "v9.9.9")
        .env("PCMP_RELEASE_BASE", format!("file://{}", stage.display()))
        .output()
        .expect("bash runs bin/install");
    (out, home.display().to_string())
}

#[test]
fn a_release_install_picks_the_aarch64_asset_for_uname_aarch64() {
    let (out, _) = release_install("rel-arm", "aarch64");
    works(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("pcmp-v9.9.9-linux-aarch64.tar.gz"), "{text}");
}

#[test]
fn a_release_install_picks_the_x86_64_asset_for_uname_x86_64() {
    let (out, _) = release_install("rel-x86", "x86_64");
    works(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("pcmp-v9.9.9-linux-x86_64.tar.gz"), "{text}");
}

#[test]
fn a_release_install_refuses_a_machine_without_a_build() {
    let (out, _) = release_install("rel-none", "riscv64");
    assert!(!out.status.success());
}
