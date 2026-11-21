use crate::{report::location, spellcheck::words};

#[derive(Debug)]
pub struct Issue {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub snippet: String,
}

trait Rule {
    fn check(&self, text: &str, issues: &mut Vec<Issue>);
}

fn note(text: &str, offset: usize, snippet: &str, message: &str, issues: &mut Vec<Issue>) {
    let (line, column) = location(text, offset);
    issues.push(Issue {
        line,
        column,
        message: message.into(),
        snippet: snippet.into(),
    });
}

struct RepeatedWords;
impl Rule for RepeatedWords {
    fn check(&self, text: &str, issues: &mut Vec<Issue>) {
        for pair in words(text).windows(2) {
            let (first_offset, first) = pair[0];
            let (second_offset, second) = pair[1];
            let between = &text[first_offset + first.len()..second_offset];
            if !between.is_empty()
                && between.chars().all(char::is_whitespace)
                && first.to_lowercase() == second.to_lowercase()
            {
                note(
                    text,
                    second_offset,
                    second,
                    "Repeated consecutive word",
                    issues,
                );
            }
        }
    }
}

struct Spacing;
impl Rule for Spacing {
    fn check(&self, text: &str, issues: &mut Vec<Issue>) {
        let chars: Vec<_> = text.char_indices().collect();
        let mut i = 0;
        while i < chars.len() {
            let (offset, ch) = chars[i];
            if ch == ' ' && i + 1 < chars.len() && chars[i + 1].1 == ' ' {
                let start = i;
                while i + 1 < chars.len() && chars[i + 1].1 == ' ' {
                    i += 1;
                }
                note(
                    text,
                    offset,
                    &text[chars[start].0..chars[i].0 + 1],
                    "Multiple spaces",
                    issues,
                );
            } else if matches!(ch, ',' | ';' | ':' | '!' | '?')
                && i + 1 < chars.len()
                && chars[i + 1].1.is_alphabetic()
            {
                let end = chars.get(i + 2).map_or(text.len(), |entry| entry.0);
                note(
                    text,
                    offset,
                    &text[offset..end],
                    "Missing space after punctuation",
                    issues,
                );
            } else if ch == '.'
                && i > 0
                && i + 1 < chars.len()
                && chars[i - 1].1.is_alphabetic()
                && chars[i + 1].1.is_alphabetic()
            {
                let end = chars.get(i + 2).map_or(text.len(), |entry| entry.0);
                note(
                    text,
                    offset,
                    &text[offset..end],
                    "Possible missing space after punctuation",
                    issues,
                );
            }
            i += 1;
        }
    }
}

struct Capitalization;
impl Rule for Capitalization {
    fn check(&self, text: &str, issues: &mut Vec<Issue>) {
        let mut start = true;
        for (offset, ch) in text.char_indices() {
            if start && ch.is_alphabetic() {
                if ch.is_lowercase() {
                    note(
                        text,
                        offset,
                        &text[offset..offset + ch.len_utf8()],
                        "Sentence should start with a capital letter",
                        issues,
                    );
                }
                start = false;
            } else if matches!(ch, '.' | '!' | '?') {
                start = true;
            } else if !ch.is_whitespace() && !matches!(ch, '"' | '\'' | '“' | '‘' | '(' | '[') {
                start = false;
            }
        }
    }
}

struct Punctuation;
impl Rule for Punctuation {
    fn check(&self, text: &str, issues: &mut Vec<Issue>) {
        let trimmed = text.trim_end();
        let ending = trimmed.trim_end_matches(['"', '\'', '”', '’', ')', ']']);
        if let Some((offset, last)) = ending.char_indices().last()
            && !matches!(last, '.' | '!' | '?')
        {
            note(
                text,
                offset,
                &ending[offset..],
                "Missing terminal punctuation",
                issues,
            );
        }
        let mut run_start = None;
        for (offset, ch) in text
            .char_indices()
            .chain(std::iter::once((text.len(), ' ')))
        {
            if matches!(ch, '.' | '!' | '?') {
                run_start.get_or_insert(offset);
            } else if let Some(start) = run_start.take()
                && offset - start >= 2
            {
                note(
                    text,
                    start,
                    &text[start..offset],
                    "Multiple consecutive punctuation marks (style)",
                    issues,
                );
            }
        }
    }
}

struct Confusions;
impl Rule for Confusions {
    fn check(&self, text: &str, issues: &mut Vec<Issue>) {
        let words = words(text);
        for pair in words.windows(2) {
            let (offset, first) = pair[0];
            let (next_offset, second) = pair[1];
            if !text[offset + first.len()..next_offset]
                .chars()
                .all(char::is_whitespace)
            {
                continue;
            }
            let first_lower = first.to_lowercase().replace('’', "'");
            let second_lower = second.to_lowercase();
            let suggestion = match (first_lower.as_str(), second_lower.as_str()) {
                ("your", "welcome") => Some("Did you mean 'you're welcome'?"),
                ("their", "is" | "are") => Some("Did you mean 'there'?"),
                ("there", "car" | "house" | "own") => Some("Did you mean 'their'?"),
                ("its", "a" | "an") => Some("Did you mean 'it's'?"),
                ("more" | "less" | "rather", "then") => Some("Did you mean 'than'?"),
                ("side", "affect") => Some("Did you mean 'side effect'?"),
                ("they're", "own") => Some("Did you mean 'their own'?"),
                _ => None,
            };
            if let Some(message) = suggestion {
                note(
                    text,
                    offset,
                    &text[offset..next_offset + second.len()],
                    message,
                    issues,
                );
            }
        }
    }
}

pub fn check(text: &str) -> Vec<Issue> {
    let rules: &[&dyn Rule] = &[
        &RepeatedWords,
        &Spacing,
        &Capitalization,
        &Punctuation,
        &Confusions,
    ];
    let mut issues = Vec::new();
    for rule in rules {
        rule.check(text, &mut issues);
    }
    issues.sort_by_key(|issue| (issue.line, issue.column));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_issues_with_locations() {
        let issues = check("Hello,world!\nthe the  cat");
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("Missing space") && (i.line, i.column) == (1, 6))
        );
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("Repeated") && (i.line, i.column) == (2, 5))
        );
        assert!(issues.iter().any(|i| i.message.contains("spaces")));
        assert!(issues.iter().any(|i| i.message.contains("capital")));
        assert!(issues.iter().any(|i| i.message.contains("terminal")));
    }

    #[test]
    fn clean_text_passes() {
        assert!(check("Hello, world! This is a test.").is_empty());
        assert!(check("").is_empty());
    }

    #[test]
    fn punctuation_and_confusions_are_notes() {
        let issues = check("Your welcome!!");
        assert!(issues.iter().any(|i| i.message.contains("you're welcome")));
        assert!(issues.iter().any(|i| i.message.contains("style")));
    }
}
