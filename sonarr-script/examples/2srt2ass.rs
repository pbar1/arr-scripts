use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use aspasia::AssSubtitle;
use aspasia::SubRipSubtitle;
use aspasia::Subtitle;
use aspasia::TextEventInterface;
use aspasia::TextSubtitle;
use aspasia::TimedSubtitleFile;
use clap::Parser;

const PREFIX: &str = "[Script Info]
Script Type: v4.00+

[Events]
Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
";

#[derive(Debug, Parser)]
struct Cli {
    /// Output subtitle file (defaults to stdout)
    #[clap(long, short, default_value = "/dev/fd/1")]
    output: PathBuf,

    /// Input subtitle file that will go on bottom
    primary: PathBuf,

    /// Input subtitle file that will go on bottom
    secondary: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut primary: AssSubtitle = TimedSubtitleFile::new(cli.primary)?.into();
    let mut secondary: AssSubtitle = TimedSubtitleFile::new(cli.secondary)?.into();

    primary.strip_formatting();
    secondary.strip_formatting();

    for event in secondary.events_mut() {
        let mut new_text = r"{\an8}".to_owned();
        new_text.push_str(&event.text);
        event.set_text(new_text);
    }

    let mut output = primary.to_string();
    output.push_str(&secondary.to_string().replace(PREFIX, ""));

    fs::write(cli.output, output)?;

    Ok(())
}
