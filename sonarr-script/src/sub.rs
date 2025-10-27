use std::collections::HashSet;
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

// TODO: Clear very short events
// TODO: Clear drawings containing \\p\d (ie, \p1)
// TODO: Clear .ass styles by name (ie, signs)

pub struct SubtitleTrack {
    inner: AssSubtitle,
}

impl SubtitleTrack {
    /// Loads a subtitle track from a file. It will automatically be converted
    /// into ASS in memory.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let inner = TimedSubtitleFile::new(path)
            .context("Failed loading subtitle file")?
            .into();
        Ok(Self { inner })
    }

    /// Saves subtitle track to an ASS file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        self.inner
            .export(path)
            .context("Failed saving subtitle file")
    }

    /// Saves subtitle track to an SRT file.
    pub fn save_srt(self, path: impl AsRef<Path>) -> Result<()> {
        let srt: SubRipSubtitle = self.inner.into();
        srt.export(path).context("Failed saving subtitle file")
    }

    /// Detects if the subtitle text events contain traditional Chinese
    /// characters.
    pub fn detect_chinese_traditional(&self) -> bool {
        let f = |text: &str| text.contains('們');
        self.inner.events().iter().any(|event| f(&event.text))
    }

    /// Detects the most frequently occurring language by events in the
    /// subtitle track.
    pub fn detect_predominant_language(&self, languages: &[Language]) -> Option<Language> {
        let detector = LanguageDetectorBuilder::from_languages(languages).build();
        let f = |text: &str| detector.detect_language_of(text);
        let languages: Counter<_> = self
            .inner
            .events()
            .iter()
            .map(|event| f(&event.text))
            .collect();
        languages.k_most_common_ordered(1).first()?.0
    }

    /// Removes formatting directives and styles.
    pub fn strip_formatting(&mut self) {
        self.inner.strip_formatting();
    }

    /// Sets the text of lines longer than some threshold to the empty string.
    pub fn clear_long_lines(&mut self, max_chars: usize) {
        self.inner.events_mut().iter_mut().for_each(|event| {
            if event.text.len() > max_chars {
                event.set_text(String::default());
            }
        })
    }

    /// Sets the text of events with rejected styles names to the empty string.
    pub fn clear_events_with_styles(&mut self, style_names: HashSet<String>) {
        for event in self.inner.events_mut() {
            if let Some(style) = &event.style
                && style_names.contains(&style.to_lowercase())
            {
                event.set_text(String::default());
            }
        }
    }

    /// Assumes that other rules have caught pretty offending style names, and
    /// so rejects the rest of their events too.
    pub fn clear_events_whose_style_has_many_existing_blanks(&mut self) {
        let styles_by_blanks: Counter<_> = self
            .inner
            .events()
            .iter()
            .filter(|event| event.text.is_empty())
            .filter_map(|event| event.style.to_owned())
            .collect();
        for event in self.inner.events_mut() {
            let Some(style) = &event.style else {
                continue;
            };
            let Some(blanks) = styles_by_blanks.get(style) else {
                continue;
            };
            if *blanks > 20 {
                event.set_text(String::default());
            }
        }
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

    #[rstest]
    #[case("../test/jjk_s02e01/extracted.en.ass")]
    #[case("../test/jjk_s02e01/extracted.zh.ass")]
    #[case("../test/jjk_s02e01/extracted.zh-TW.ass")]
    fn test_helper(#[case] path: &str) {
        let out = path.replace("extracted", "cleaned");
        let out = PathBuf::from_str(&out).unwrap();
        let path = PathBuf::from_str(path).unwrap();
        let mut subtitle = SubtitleTrack::load(&path).unwrap();
        subtitle.strip_formatting();
        // subtitle.clear_events_with_styles(HashSet::from_iter(["signs".to_owned()]));
        subtitle.clear_long_lines(140);
        subtitle.clear_events_whose_style_has_many_existing_blanks();
        subtitle.save(&out).unwrap();
    }
}
