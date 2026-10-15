use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use pito_compare::{Chart, Kind};

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
The README at https://github.com/gmrdad82/pito-compare has every field.";

#[derive(Parser)]
#[command(
    name = "pcmp",
    bin_name = "pcmp",
    version,
    about = "Draws comparison charts as PNGs from a JSON or TOML data file",
    after_help = AFTER
)]
struct Cli {
    #[arg(value_name = "DATA", help = "the data file, .json or .toml")]
    data: PathBuf,
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

fn run(cli: &Cli) -> Result<Vec<PathBuf>, String> {
    let chart = Chart::load(&cli.data).map_err(|e| e.to_string())?;
    let stem = cli
        .data
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
        None => format!("{}: {e}", cli.data.display()),
    })?;
    Ok(written)
}

fn print(paths: &[PathBuf]) {
    let mut out = std::io::stdout().lock();
    for p in paths {
        let shown = p.strip_prefix(".").unwrap_or(p);
        let shown: &Path = if shown.as_os_str().is_empty() {
            p
        } else {
            shown
        };
        if writeln!(out, "{}", shown.display()).is_err() {
            return;
        }
    }
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
    let cli = Cli::parse();
    match run(&cli) {
        Ok(paths) => {
            print(&paths);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pcmp: {e}");
            ExitCode::from(1)
        }
    }
}
