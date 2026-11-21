use std::collections::HashSet;
use symspell::{SymSpell, UnicodeStringStrategy, Verbosity};

const DICTIONARY: &str = include_str!("../data/frequency_dictionary_en.txt");

#[derive(Debug)]
pub struct Issue {
    pub word: String,
    pub line: usize,
    pub column: usize,
    pub suggestions: Vec<String>,
}

pub struct SpellChecker {
    dictionary: SymSpell<UnicodeStringStrategy>,
}

// Offsets are byte offsets into the original UTF-8 text.
pub fn words(text: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let mut start = None;
    for (offset, ch) in text.char_indices() {
        if ch.is_alphabetic() || ((ch == '\'' || ch == '’') && start.is_some()) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            let word = text[begin..offset].trim_end_matches(['\'', '’']);
            if !word.is_empty() {
                result.push((begin, word));
            }
        }
    }
    if let Some(begin) = start {
        let word = text[begin..].trim_end_matches(['\'', '’']);
        if !word.is_empty() {
            result.push((begin, word));
        }
    }
    result
}

impl SpellChecker {
    pub fn new() -> Self {
        let mut dictionary = SymSpell::default();
        for line in DICTIONARY.lines() {
            dictionary.load_dictionary_line(line.trim_start_matches('\u{feff}'), 0, 1, " ");
        }
        Self { dictionary }
    }

    pub fn check(&self, text: &str, ignored: &HashSet<String>) -> Vec<Issue> {
        words(text)
            .into_iter()
            .filter_map(|(offset, word)| {
                let normalized = word.to_lowercase().replace('’', "'");
                if ignored.contains(&word.to_lowercase())
                    || !self
                        .dictionary
                        .lookup(&normalized, Verbosity::Top, 0)
                        .is_empty()
                {
                    return None;
                }
                let suggestions = self
                    .dictionary
                    .lookup(&normalized, Verbosity::Closest, 2)
                    .into_iter()
                    .take(3)
                    .map(|suggestion| suggestion.term)
                    .collect();
                let (line, column) = crate::report::location(text, offset);
                Some(Issue {
                    word: word.into(),
                    line,
                    column,
                    suggestions,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn checker() -> &'static SpellChecker {
        static CHECKER: OnceLock<SpellChecker> = OnceLock::new();
        CHECKER.get_or_init(SpellChecker::new)
    }

    #[test]
    fn finds_misspelling_and_suggests_correction() {
        let issues = checker().check("Hello, wurld!", &HashSet::new());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].word, "wurld");
        assert!(issues[0].suggestions.iter().any(|word| word == "world"));
    }

    #[test]
    fn accepts_good_text_and_personal_words() {
        assert!(checker().check("Hello world!", &HashSet::new()).is_empty());
        assert!(
            checker()
                .check("Wurld", &HashSet::from(["wurld".into()]))
                .is_empty()
        );
    }
}
