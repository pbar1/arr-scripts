use std::path::PathBuf;

use tracing::info;

use crate::subtitle::{self};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Primary subtitle (bottom)
    pub primary: PathBuf,

    /// Secondary subtitle (top)
    pub secondary: PathBuf,
}

impl Args {
    pub fn run(&self) -> anyhow::Result<()> {
        // FIXME: This is destructive
        subtitle::clean_srt_file(&self.primary)?;
        subtitle::clean_srt_file(&self.secondary)?;
        info!("cleaned input subtitles");

        let _merged = subtitle::merge_srt(&self.primary, &self.secondary)?;
        info!("mergend subtitles");

        Ok(())
    }
}
