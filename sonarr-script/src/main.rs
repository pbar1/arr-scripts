use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;
use crate::cli::SubCommand;

mod cli;
mod subtitle;

fn main() -> anyhow::Result<ExitCode> {
    let format = tracing_subscriber::fmt::format();
    tracing_subscriber::fmt().event_format(format).init();

    match Cli::parse() {
        Cli::SonarrSubtitleMerge(args) | Cli::Default(SubCommand::SonarrSubtitleMerge(args)) => {
            args.run()
        }
        Cli::SubtitleMerge(args) | Cli::Default(SubCommand::SubtitleMerge(args)) => args.run(),
    }
    .map(|_| ExitCode::SUCCESS)
}
