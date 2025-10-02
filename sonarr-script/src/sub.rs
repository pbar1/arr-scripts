use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use aspasia::Subtitle;
use aspasia::TextSubtitle;
use aspasia::TimedSubtitleFile;
use counter::Counter;
pub use lingua::Language;
use lingua::LanguageDetectorBuilder;

macro_rules! timed_subtitle {
    // immutable case
    ($inner:expr, |$s:ident| $body:expr) => {
        match $inner {
            TimedSubtitleFile::Ssa($s) => $body,
            TimedSubtitleFile::Ass($s) => $body,
            TimedSubtitleFile::SubRip($s) => $body,
            TimedSubtitleFile::WebVtt($s) => $body,
            TimedSubtitleFile::MicroDvd($s) => $body,
        }
    };
    // mutable case
    (&mut $inner:expr, |$s:ident| $body:expr) => {
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
    pub fn load(path: &Path) -> Result<Self> {
        let inner = TimedSubtitleFile::new(path).context("Failed loading subtitle file")?;
        Ok(Self { inner })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.inner
            .export(path)
            .context("Failed saving subtitle file")
    }

    pub fn detect_chinese_traditional(&self) -> bool {
        let f = |text: &str| text.contains('們');
        timed_subtitle!(&self.inner, |s| s.events().iter().any(|e| f(&e.text)))
    }

    pub fn detect_predominant_language(&self, languages: &[Language]) -> Option<Language> {
        let detector = LanguageDetectorBuilder::from_languages(languages).build();
        let f = |text: &str| detector.detect_language_of(text);
        let languages: Counter<_> = timed_subtitle!(&self.inner, |s| s
            .events()
            .iter()
            .map(|e| f(&e.text))
            .collect());
        languages.k_most_common_ordered(1).first()?.0
    }

    pub fn strip_formatting(&mut self) {
        timed_subtitle!(&mut self.inner, |s| s.strip_formatting());
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

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
}
