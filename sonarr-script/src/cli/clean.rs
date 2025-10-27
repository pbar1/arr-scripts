use std::fs;
use std::str::FromStr;

use aspasia::SubRipSubtitle;
use aspasia::Subtitle;
use camino::Utf8PathBuf;

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Input subtitle file
    input: Utf8PathBuf,

    // TODO: Support in-place
    /// Output subtitle file
    #[clap(long, short, default_value = "/dev/fd/1")]
    output: Utf8PathBuf,

    /// Remove lines whose length is over this limit
    #[clap(long)]
    length: Option<usize>,

    /// Remove lines whose characters per second (CPS) is over this limit
    #[clap(long)]
    cps: Option<usize>,
}

impl Args {
    pub fn run(&self) -> anyhow::Result<()> {
        // Seems like reading manually like this is needed to avoid having
        // formatting getting stripped
        let input = fs::read_to_string(&self.input)?;
        let mut events = SubRipSubtitle::from_str(&input)?.events().to_owned();

        // Delete long lines
        if let Some(length_limit) = self.length {
            events.retain(|event| event.text.len() < length_limit);
        }

        // Delete lines with CPS too high
        if let Some(cps_limit) = self.cps {
            events.retain(|event| {
                let duration: usize = (event.end.seconds() - event.start.seconds())
                    .try_into()
                    .unwrap_or(1);
                let duration = if duration == 0 { 1 } else { duration };
                let characters = event.text.chars().count();
                let cps = characters / duration;
                cps < cps_limit
            });
        }

        let output = SubRipSubtitle::from_events(events).to_string();
        fs::write(&self.output, output)?;

        Ok(())
    }
}
