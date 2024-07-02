use std::path::PathBuf;

use anyhow::Context;
use tracing::info;

use crate::subtitle::SubtitleMergeContext;
use crate::subtitle::{self};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Sonarr event type
    #[clap(long, env = "sonarr_eventtype", default_value = "Download")]
    pub eventtype: EventType,

    /// `True` when an existing file is upgraded, `False` otherwise
    #[clap(long, env = "sonarr_isupgrade")]
    pub isupgrade: Option<String>,

    /// Full path to the episode file
    #[clap(short = 'i', long, env = "sonarr_episodefile_path")]
    pub episodefile_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "pascal_case")]
#[non_exhaustive]
pub enum EventType {
    Test,
    Download,
}

impl Args {
    pub fn run(&self) -> anyhow::Result<()> {
        match self.eventtype {
            EventType::Test => self.handle_test(),
            EventType::Download => self.handle_download(),
        }
    }

    fn handle_test(&self) -> anyhow::Result<()> {
        info!("test event, exiting");
        Ok(())
    }

    fn handle_download(&self) -> anyhow::Result<()> {
        let media_file = self
            .episodefile_path
            .clone()
            .context("sonarr_episodefile_path must be set")?;

        let context = SubtitleMergeContext { media_file };

        subtitle::extract_and_merge(&context)
    }
}
