use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

fn path() -> io::Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("APPDATA")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".config"))
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot locate config directory"))?;
    Ok(base.join("spellcheck-cli").join("ignore.txt"))
}

pub fn load() -> io::Result<HashSet<String>> {
    match fs::read_to_string(path()?) {
        Ok(contents) => Ok(contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_lowercase)
            .collect()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(HashSet::new()),
        Err(err) => Err(err),
    }
}

pub fn add_word(word: &str) -> io::Result<()> {
    let tokens = crate::spellcheck::words(word);
    if tokens.len() != 1 || tokens[0].0 != 0 || tokens[0].1.len() != word.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a single word",
        ));
    }
    if load()?.contains(&word.to_lowercase()) {
        return Ok(());
    }
    let path = path()?;
    fs::create_dir_all(path.parent().expect("config path has a parent"))?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    // A user-edited file may not end in a newline.
    if file.metadata()?.len() > 0 {
        writeln!(file)?;
    }
    writeln!(file, "{word}")?;
    Ok(())
}
