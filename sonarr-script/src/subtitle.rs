use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use aspasia::SubRipSubtitle;
use aspasia::Subtitle;
use aspasia::TextEvent;
use aspasia::TextEventInterface;
use aspasia::TextSubtitle;
use aspasia::TimedSubtitleFile;
use aspasia::subrip::SubRipEvent;
use regex::Regex;
use tracing::info;

pub struct SubtitleMergeContext {
    pub media_file: PathBuf,
}

pub fn extract_and_merge(context: &SubtitleMergeContext) -> anyhow::Result<()> {
    let media_file = &context.media_file;

    info!(media_file = %media_file.to_string_lossy(), "download event");

    let media_file_stem = media_file
        .file_stem()
        .context("unable to get media file stem")?;
    let subtitle_dir = media_file
        .parent()
        .context("unable to get media file")?
        .join(".subtitles")
        .join(media_file_stem);
    std::fs::create_dir_all(&subtitle_dir)?;

    let lang_filter = HashSet::from(["zh", "en"]);
    let codec_filter = HashSet::from(["srt", "subrip", "ass", "ssa", "mov_text", "webvtt", "ttml"]); // ffmpeg -codecs

    let subtitle_streams = get_subtitle_streams(media_file)?;
    for ref s in subtitle_streams {
        info!(stream_id = %s.stream_id, language_code = %s.language_code, codec = %s.codec, "found subtitle stream");
        if !lang_filter.contains(s.language_code.as_str())
            || !codec_filter.contains(s.codec.as_str())
        {
            continue;
        }

        info!(stream_id = %s.stream_id, language_code = %s.language_code, codec = %s.codec, "dumping subtitle file");
        let dumped = dump_subtitle_file(s, &subtitle_dir)?;

        info!(file = %dumped.to_string_lossy(), "cleaning subtitle file");
        clean_subtitle_file(&dumped)?;

        if s.language_code == "zh" {
            info!(file = %dumped.to_string_lossy(), "ensuring chinese character classification");
            ensure_hanzi(&dumped)?;
        }
    }

    let Some(en) = get_best_srt_en(&subtitle_dir) else {
        return Ok(());
    };
    let live_en = media_file.with_extension("en.srt");
    std::fs::copy(&en, live_en)?;

    // FIXME: Extract the repeated extension stuff
    if let Some(chs) = get_best_srt_chs(&subtitle_dir) {
        let merged = merge_subtitle_files(&chs, &en)?;
        let live_chs = media_file.with_extension("zh.srt");
        std::fs::copy(&merged, live_chs)?;
    }
    if let Some(cht) = get_best_srt_cht(&subtitle_dir) {
        let merged = merge_subtitle_files(&cht, &en)?;
        let live_cht = media_file.with_extension("zh-TW.srt");
        std::fs::copy(&merged, live_cht)?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct SubtitleStream {
    pub source_file: PathBuf,
    pub stream_id: String,
    pub language_code: String,
    pub codec: String,
}

fn get_subtitle_streams(media_file: impl AsRef<Path>) -> anyhow::Result<Vec<SubtitleStream>> {
    let re =
        Regex::new(r"Stream #(?<stream>\d+:\d+).*?\((?<lang>\w+)\).*?Subtitle: (?<codec>\w+)")?;

    let output = std::process::Command::new("ffprobe")
        .args(["-i", &media_file.as_ref().to_string_lossy()])
        .output()?;
    let mediainfo = String::from_utf8(output.stderr)?;

    let subtitle_streams: Vec<SubtitleStream> = re
        .captures_iter(&mediainfo)
        .flat_map(|capture| {
            let stream_id = capture.name("stream")?;
            let language_code = capture.name("lang")?;
            let codec = capture.name("codec")?;
            let language_code = map_language_code(language_code.as_str());
            Some(SubtitleStream {
                source_file: PathBuf::from(media_file.as_ref()),
                stream_id: stream_id.as_str().to_string(),
                language_code,
                codec: codec.as_str().to_string(),
            })
        })
        .collect();

    Ok(subtitle_streams)
}

fn dump_subtitle_file(
    subtitle_stream: &SubtitleStream,
    destination_dir: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let stream = subtitle_stream.stream_id.replace(":", "_");
    let lang = &subtitle_stream.language_code;
    let name = format!("{stream}.{lang}.srt");
    let sub_file = destination_dir.as_ref().join(name);

    let _output = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            &subtitle_stream.source_file.to_string_lossy(),
            "-map",
            &subtitle_stream.stream_id,
            "-c:s",
            "srt",
            &sub_file.to_string_lossy(),
        ])
        .output()?;

    Ok(sub_file)
}

fn map_language_code(input: &str) -> String {
    match input {
        "zh" | "zho" | "chi" => "zh".into(),
        "en" | "eng" => "en".into(),
        other => other.into(),
    }
}

pub fn clean_subtitle_file(subtitle_file: &Path) -> Result<()> {
    let mut subtitle = TimedSubtitleFile::new(subtitle_file)
        .context("error opening subtitle file for cleaning")?;

    match &mut subtitle {
        TimedSubtitleFile::Ssa(s) => s.strip_formatting(),
        TimedSubtitleFile::Ass(s) => s.strip_formatting(),
        TimedSubtitleFile::SubRip(s) => s.strip_formatting(),
        TimedSubtitleFile::WebVtt(s) => s.strip_formatting(),
        TimedSubtitleFile::MicroDvd(s) => s.strip_formatting(),
    }

    subtitle
        .export(subtitle_file)
        .context("error writing cleaned subtitle file")?;

    Ok(())
}

pub fn ensure_hanzi(srt_file: impl AsRef<Path>) -> anyhow::Result<()> {
    let srt_file = srt_file.as_ref();
    let srt = SubRipSubtitle::from_path(srt_file)?;
    for event in srt.events() {
        if event.as_plaintext().contains("們") {
            let new_file = srt_file.to_string_lossy().replace(".zh.srt", ".zh-TW.srt");
            std::fs::rename(srt_file, new_file)?;
            break;
        }
    }
    Ok(())
}

// TODO: Find the largest instead of just the first
fn get_best_srt(subtitle_dir: impl AsRef<Path>, suffix: &str) -> anyhow::Result<PathBuf> {
    let paths = std::fs::read_dir(subtitle_dir.as_ref())?;
    for path in paths {
        let path = path?;
        if path.file_name().to_string_lossy().ends_with(suffix) {
            return Ok(path.path());
        }
    }
    bail!("unable to find an srt with suffix: {suffix}");
}

fn get_best_srt_en(subtitle_dir: impl AsRef<Path>) -> Option<PathBuf> {
    get_best_srt(subtitle_dir, ".en.srt").ok()
}

fn get_best_srt_chs(subtitle_dir: impl AsRef<Path>) -> Option<PathBuf> {
    get_best_srt(subtitle_dir, ".zh.srt").ok()
}

fn get_best_srt_cht(subtitle_dir: impl AsRef<Path>) -> Option<PathBuf> {
    get_best_srt(subtitle_dir, ".zh-TW.srt").ok()
}

pub fn merge_subtitle_files(bottom: &Path, top: &Path) -> Result<PathBuf> {
    info!(
        bottom = %bottom.to_string_lossy(),
        top = %top.to_string_lossy(),
        "Merging subtitle files"
    );
    let output = bottom.with_extension("merged.srt");

    // Converts non-SRT into SRT format in memory
    let bottom_subs: SubRipSubtitle = TimedSubtitleFile::new(bottom)
        .context("error loading bottom subtitle file")?
        .into();
    let mut top_subs: SubRipSubtitle = TimedSubtitleFile::new(top)
        .context("error loading top subtitle file")?
        .into();

    for event in top_subs.events_mut() {
        // move text to top screen with this formatting directive
        let mut text = String::from(r"{\an8}");
        text.push_str(&event.text);
        event.set_text(text)
    }

    let mut output_events: Vec<SubRipEvent> = Vec::new();
    output_events.extend_from_slice(bottom_subs.events());
    output_events.extend_from_slice(top_subs.events());
    output_events.sort_by(|a, b| a.start.cmp(&b.start));
    let mut output_srt = SubRipSubtitle::from_events(output_events);
    output_srt.renumber();

    output_srt.export(&output)?;
    info!(output = %output.to_string_lossy(), "Wrote merged subtitle file");

    Ok(output)
}
