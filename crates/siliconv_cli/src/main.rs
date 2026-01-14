//! CLI interface for Siliconv.

use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use siliconv_formats::DynamicReplay;
use tracing::level_filters::LevelFilter;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the replay file to process
    #[arg(short, long)]
    input: Vec<PathBuf>,

    #[arg(short, long)]
    format: Option<String>,

    output: String,
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .without_time()
        .init();

    let args = Cli::parse();

    for path in args.input {
        let ext = path
            .extension()
            .map_or_else(String::new, |s| s.to_string_lossy().to_string());

        let mut file = BufReader::new(File::open(&path).expect("failed to open file"));
        tracing::info!("opening {} for reading", &path.display());

        let start = std::time::Instant::now();
        let replay = DynamicReplay::read(&mut file, &ext).expect("failed to read replay");

        tracing::info!(
            "[took {}ms] read {} inputs from {:?} replay at {}",
            start.elapsed().as_millis(),
            replay.0.actions.len(),
            replay.0.format,
            path.display(),
        );
    }
}
