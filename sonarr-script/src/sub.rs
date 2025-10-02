use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use aspasia::AssSubtitle;
use aspasia::SubRipSubtitle;
use aspasia::Subtitle;
use aspasia::TextEventInterface;
use aspasia::TextSubtitle;
use aspasia::TimedSubtitleFile;
use counter::Counter;
pub use lingua::Language;
use lingua::LanguageDetectorBuilder;

macro_rules! timed_subtitle {
    ($inner:expr, |$s:ident| $body:expr) => {
        match $inner {
            TimedSubtitleFile::Ssa($s) => $body,
            TimedSubtitleFile::Ass($s) => $body,
            TimedSubtitleFile::SubRip($s) => $body,
            TimedSubtitleFile::WebVtt($s) => $body,
            TimedSubtitleFile::MicroDvd($s) => $body,
        }
    };
}

pub struct SubtitleTrack {
    inner: TimedSubtitleFile,
}

impl SubtitleTrack {
    /// Loads a subtitle track from a file.
    pub fn load(path: &Path) -> Result<Self> {
        let inner = TimedSubtitleFile::new(path).context("Failed loading subtitle file")?;
        Ok(Self { inner })
    }

    /// Saves a subtitle track to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.inner
            .export(path)
            .context("Failed saving subtitle file")
    }

    /// Detects if the subtitle text events contain traditional Chinese
    /// characters.
    pub fn detect_chinese_traditional(&self) -> bool {
        let f = |text: &str| text.contains('們');
        timed_subtitle!(&self.inner, |subtitle| subtitle
            .events()
            .iter()
            .any(|event| f(&event.text)))
    }

    /// Detects the most frequently occurring language by events in the
    /// subtitle track.
    pub fn detect_predominant_language(&self, languages: &[Language]) -> Option<Language> {
        let detector = LanguageDetectorBuilder::from_languages(languages).build();
        let f = |text: &str| detector.detect_language_of(text);
        let languages: Counter<_> = timed_subtitle!(&self.inner, |subtitle| subtitle
            .events()
            .iter()
            .map(|event| f(&event.text))
            .collect());
        languages.k_most_common_ordered(1).first()?.0
    }

    /// Removes formatting directives and styles.
    pub fn strip_formatting(&mut self) {
        timed_subtitle!(&mut self.inner, |subtitle| subtitle.strip_formatting());
    }

    /// Sets the text of lines longer than some threshold to the empty string.
    pub fn clear_long_lines(&mut self, max_chars: usize) {
        timed_subtitle!(&mut self.inner, |subtitle| subtitle
            .events_mut()
            .iter_mut()
            .for_each(|event| {
                if event.text.len() > max_chars {
                    event.set_text(String::default());
                }
            }))
    }

    /// Convert the subtitle track into `SRT` format.
    pub fn into_srt(self) -> Self {
        let inner: SubRipSubtitle = self.inner.into();
        Self {
            inner: TimedSubtitleFile::SubRip(inner),
        }
    }

    /// Convert the subtitle track into `ASS` format.
    pub fn into_ass(self) -> Self {
        let inner: AssSubtitle = self.inner.into();
        Self {
            inner: TimedSubtitleFile::Ass(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::sub;

    #[rstest]
    #[case("../test/jjk_s02e01/extracted.en.ass", Language::English)]
    #[case("../test/jjk_s02e01/extracted.zh.ass", Language::Chinese)]
    #[case("../test/jjk_s02e01/extracted.zh-TW.ass", Language::Chinese)]
    fn test_detect_predominant_language(
        #[case] path: &str,
        #[case] language_should: impl Into<Option<Language>>,
    ) {
        let path = PathBuf::from_str(path).unwrap();
        let language_should = language_should.into();

        let subtitle = SubtitleTrack::load(&path).unwrap();
        let language =
            subtitle.detect_predominant_language(&[Language::English, Language::Chinese]);

        assert_eq!(language, language_should);
    }

    #[rstest]
    #[case("../test/jjk_s02e01/extracted.en.ass", false)]
    #[case("../test/jjk_s02e01/extracted.zh.ass", false)]
    #[case("../test/jjk_s02e01/extracted.zh-TW.ass", true)]
    fn test_detect_chinese_traditional(#[case] path: &str, #[case] should: bool) {
        let path = PathBuf::from_str(path).unwrap();

        let subtitle = SubtitleTrack::load(&path).unwrap();
        let got = subtitle.detect_chinese_traditional();

        assert_eq!(got, should);
    }

    #[rstest]
    #[case("../test/jjk_s02e01/extracted.en.ass")]
    #[case("../test/jjk_s02e01/extracted.zh.ass")]
    #[case("../test/jjk_s02e01/extracted.zh-TW.ass")]
    fn test_helper(#[case] path: &str) {
        let out = format!("{path}.test");
        let out = PathBuf::from_str(&out).unwrap();
        let path = PathBuf::from_str(path).unwrap();
        let mut subtitle = SubtitleTrack::load(&path).unwrap();
        subtitle.strip_formatting();
        subtitle.clear_long_lines(140);
        subtitle.into_ass().save(&out).unwrap();
    }
}
