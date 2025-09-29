use aspasia::AssSubtitle;
use aspasia::Subtitle;
use lingua::Language;
use lingua::LanguageDetector;
use lingua::LanguageDetectorBuilder;

const BASE_PATH: &str = "/Users/pierce/code/arr-scripts/working";
const SUBTITLE_PATH: &str = "Jujutsu Kaisen (2020) - S01E01 - 001 [WEBDL-1080p][10bit][x265][AAC 2.0][JA]-Judas.mul-x-zt-en.ass";

fn detect_and_show(detector: &LanguageDetector, text: &str) -> Option<Language> {
    let language = detector.detect_language_of(text);
    println!("{language:?} --- {text}");
    language
}

fn main() -> anyhow::Result<()> {
    let path = format!("{BASE_PATH}/{SUBTITLE_PATH}");
    let mut subtitle = AssSubtitle::from_path(&path)?;

    let detector =
        LanguageDetectorBuilder::from_languages(&[Language::English, Language::Chinese]).build();

    for event in subtitle.events_mut() {
        let language = detect_and_show(&detector, &event.text);

        use Language as L;
        match language {
            Some(L::English) => {
                event.style = Some("Latin-Top".to_owned());
            }
            Some(L::Chinese) => {
                event.style = Some("CJK-Bottom".to_owned());
            }
            _ => {}
        }
    }

    subtitle.export(&path)?;

    Ok(())
}
