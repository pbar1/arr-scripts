mod sonarr_subtitle_merge;
mod subtitle_merge;

/// This is a multicall binary like BusyBox. For example, if the program is
/// symlinked to the name of a subcommand, that subcommand will be executed.
#[derive(Debug, clap::Parser)]
#[clap(multicall = true)]
pub enum Cli {
    // Name here must be the default binary basename
    #[clap(subcommand, name = "sonarr-script")]
    Default(SubCommand),
    SonarrSubtitleMerge(sonarr_subtitle_merge::Args),
    SubtitleMerge(subtitle_merge::Args),
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    /// Sonarr Custom Script to create dual-language subtitles
    SonarrSubtitleMerge(sonarr_subtitle_merge::Args),

    /// Merge two subtitle files
    SubtitleMerge(subtitle_merge::Args),
}
