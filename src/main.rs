use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use pito_compare::{Chart, Kind};

mod skill;

const AFTER: &str = "\
The data file is JSON or TOML (by its extension, else by its first character):

  title = \"DECODE SPEED\"
  unit = \"FRAMES PER SECOND\"
  better = \"higher\"                  higher or lower
  subtitle = \"RYZEN 9 7950X\"          optional, like date, notes, suffix
  date = \"2026-10-11\"
  suffix = \" FPS\"
  notes = [\"* NOT LIKE FOR LIKE.\"]
  parties = [{ name = \"OURS\" }, { name = \"THEIRS\" }]

  [[groups]]
  name = \"CPU\"
  rows = [{ label = \"H.264 1080P\", values = [412, 298], mark = \"*\" }]

Each PNG is written to <out>/<data file's stem>-<kind>.png and its path printed.
With --kind all, one party draws bars; two or more draw the paired and ratio charts.
The README at https://github.com/gmrdad82/pito-compare has every field.

pcmp comes with a skill for AI coding agents. `pcmp skill install` puts it in
~/.claude/skills/pcmp and ~/.agents/skills/pcmp as files.";

#[derive(Parser)]
#[command(
    name = "pcmp",
    bin_name = "pcmp",
    version,
    about = "Draws comparison charts as PNGs from a JSON or TOML data file",
    after_help = AFTER,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(
        value_name = "DATA",
        required = true,
        help = "the data file, .json or .toml"
    )]
    data: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "DIR",
        default_value = ".",
        help = "where the PNGs go, made if missing"
    )]
    out: PathBuf,
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = KindArg::All,
        help = "which charts to draw"
    )]
    kind: KindArg,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "install the pcmp skill for Claude Code and other agents, or check it")]
    Skill {
        #[command(subcommand)]
        action: skill::Action,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum KindArg {
    All,
    Paired,
    Ratio,
    Bars,
}

impl From<KindArg> for Kind {
    fn from(k: KindArg) -> Self {
        match k {
            KindArg::All => Kind::All,
            KindArg::Paired => Kind::Paired,
            KindArg::Ratio => Kind::Ratio,
            KindArg::Bars => Kind::Bars,
        }
    }
}

fn run(cli: &Cli, data: &Path) -> Result<Vec<PathBuf>, String> {
    let chart = Chart::load(data).map_err(|e| e.to_string())?;
    let stem = data
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("chart");
    let mut written = Vec::new();
    let mut failed = None;
    pito_compare::render_each(&chart, cli.kind.into(), |name, canvas| {
        let path = cli.out.join(format!("{stem}-{name}.png"));
        canvas
            .save(&path)
            .inspect_err(|e| failed = Some(e.clone()))?;
        written.push(path);
        Ok(())
    })
    .map_err(|e| match failed.take() {
        Some(e) => e.to_string(),
        None => format!("{}: {e}", data.display()),
    })?;
    Ok(written)
}

fn shown(p: &Path) -> &Path {
    let stripped = p.strip_prefix(".").unwrap_or(p);
    if stripped.as_os_str().is_empty() {
        p
    } else {
        stripped
    }
}

fn print(lines: &[String]) {
    let mut out = std::io::stdout().lock();
    for line in lines {
        if writeln!(out, "{line}").is_err() {
            return;
        }
    }
}

const FORMS: [(&str, &str); 4] = [
    (
        "pcmp <data> [--out <dir>] [--kind <kind>]",
        "draw the charts of a .json or .toml file: all, paired, ratio or bars",
    ),
    (
        "pcmp skill install|status",
        "install the skill pcmp comes with for Claude Code and other agents, or check it",
    ),
    (
        "pcmp --help",
        "the full help, with every option and the data format",
    ),
    ("pcmp --version", "print the version"),
];

fn summary() -> String {
    let width = FORMS.iter().map(|(f, _)| f.len()).max().unwrap_or(0) + 2;
    let mut s = format!(
        "pcmp {}: draws comparison charts as PNGs from a JSON or TOML data file\n\n",
        env!("CARGO_PKG_VERSION")
    );
    for (form, what) in FORMS {
        s.push_str(&format!("  {form:<width$}{what}\n"));
    }
    s.push_str(
        "\nEach PNG is written to <dir>/<data file's stem>-<kind>.png and its path printed.\n",
    );
    s
}

fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown".into());
        eprintln!("pcmp: unexpected failure ({what}); please report it as an issue");
        std::process::exit(1);
    }));
    if std::env::args_os().len() == 1 {
        let _ = std::io::stdout().lock().write_all(summary().as_bytes());
        return ExitCode::SUCCESS;
    }
    let cli = Cli::parse();
    let done = match (&cli.command, &cli.data) {
        (Some(Command::Skill { action }), _) => skill::run(action),
        (None, Some(data)) => run(&cli, data).map(|paths| {
            paths
                .iter()
                .map(|p| shown(p).display().to_string())
                .collect()
        }),
        (None, None) => Err("no data file".to_owned()),
    };
    match done {
        Ok(lines) => {
            print(&lines);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pcmp: {e}");
            ExitCode::from(1)
        }
    }
}
