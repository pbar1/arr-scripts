use anyhow::Context;
use aspasia::AssSubtitle;
use aspasia::Subtitle;
use aspasia::TimedSubtitleFile;
use camino::Utf8PathBuf;
use tracing::debug;
use tracing::info;

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    input: Utf8PathBuf,

    #[clap(long, short)]
    output: Option<Utf8PathBuf>,

    #[clap(long, short)]
    format: Option<Format>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Format {
    Srt,
    Ass,
}

impl From<Format> for aspasia::Format {
    fn from(value: Format) -> Self {
        match value {
            Format::Srt => aspasia::Format::SubRip,
            Format::Ass => aspasia::Format::Ass,
        }
    }
}

impl Args {
    pub fn run(&self) -> anyhow::Result<()> {
        debug!(?self, "Got args");

        let input_format =
            aspasia::detect_format(&self.input).context("Failed detecting subtitle format")?;
        info!(?input_format, "Detected subtitle format");

        let input_subtitle =
            TimedSubtitleFile::new(&self.input).context("Failed reading timed subtitle file")?;

        let output_format = self
            .format
            .clone()
            .map(aspasia::Format::from)
            .unwrap_or(input_format);

        match output_format {
            aspasia::Format::Ass => {
                let o = AssSubtitle::from(input_subtitle);
                o.export(self.output.clone().unwrap())?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}
