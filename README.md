# spellcheck-cli

A standalone, offline English spelling and grammar checker for drafts, emails,
and chat messages. Proofread without spending AI tokens or sending your text to
a service. It uses a bundled English frequency dictionary and simple grammar
rules, not an AI model; suggestions are not guaranteed to be correct.

## Install

Install Rust with [rustup](https://rustup.rs/), then run from this directory:

```sh
cargo install --path . --locked
```

Alternatively, run `cargo build --release` and use `target/release/spellcheck`.
Building needs the dependencies available locally or downloaded by Cargo; the
installed binary needs no network connection or external dictionary.

## Usage

```sh
echo "Hello, wurld!" | spellcheck
cat draft.txt | spellcheck --no-color
spellcheck check draft.txt
spellcheck add-word MyProjectName
```

Run `spellcheck` with no arguments in a terminal for interactive mode. Enter
or paste text one line at a time; each nonempty line is checked immediately.
Press Ctrl+D (Ctrl+Z then Enter on Windows) to exit. Piped input and files are
checked as a whole, including across lines. Files and input must be UTF-8.

Reports include one-based line and Unicode-character column numbers, up to
three spelling suggestions, grammar/style notes, and totals. Color is used on
terminals; `--no-color` disables it. The exit code is 0 for clean text, 1
when issues are found (including in any interactive entry), and 2 for an
input/configuration error.

The personal dictionary is a UTF-8 file with one word per line at
`$XDG_CONFIG_HOME/spellcheck-cli/ignore.txt`, or
`~/.config/spellcheck-cli/ignore.txt` if `XDG_CONFIG_HOME` is unset. On
Windows, `%APPDATA%\spellcheck-cli\ignore.txt` is used when available. Edit
it directly or use `add-word`; matching is case-insensitive.

## Limitations and dictionary

The grammar checker uses heuristics, so it can miss errors or report false
positives, especially for URLs, abbreviations, and names. Confusion notes are
limited to a few recognizable phrases and should be reviewed in context.
The bundled frequency list contains common words and names, rather than a
complete authoritative dictionary, and may include infrequently used forms.

The English frequency dictionary comes from
[SymSpell](https://github.com/wolfgarbe/SymSpell), and its MIT license is
included in `data/SymSpell-LICENSE`.
