extern crate clap;
extern crate regex;

use clap::{App, Arg};
use regex::Regex;
//use std::collections::HashMap;
//use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    query: Vec<String>,
}

type MyResult<T> = Result<T, Box<dyn Error>>;

// --------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    let matches = App::new("run_pear")
        .version("0.1.0")
        .author("Ken Youens-Clark")
        .about("Runs Pear")
        .arg(
            Arg::with_name("query")
                .short("q")
                .long("query")
                .value_name("FILE_OR_DIR")
                .help("File input or directory")
                .required(true)
                .min_values(1),
        )
        .get_matches();

    Ok(Config {
        query: matches.values_of_lossy("query").unwrap(),
    })
}

// --------------------------------------------------
pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.query)?;

    if files.is_empty() {
        let msg = format!("No input files from query \"{:?}\"", &config.query);
        return Err(From::from(msg));
    }

    println!(
        "Will process {} file{}",
        files.len(),
        if files.len() == 1 { "" } else { "s" }
    );

    let _x = classify(&files);

    Ok(())
}

// --------------------------------------------------
fn find_files(paths: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = vec![];
    for path in paths {
        let meta = fs::metadata(path)?;
        if meta.is_file() {
            files.push(path.to_owned());
        } else {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                if meta.is_file() {
                    files.push(entry.path().display().to_string());
                }
            }
        };
    }

    if files.is_empty() {
        return Err(From::from("No input files"));
    }

    Ok(files)
}

// --------------------------------------------------
fn classify(paths: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    let paths = paths.into_iter().map(Path::new);
    let mut exts: Vec<String> = paths
        .clone()
        .into_iter()
        .map(get_extension)
        .filter_map(|x| x)
        .collect();
    exts.dedup();

    let dots = Regex::new(r"\.").unwrap();
    let exts: Vec<String> = exts
        .into_iter()
        .map(|x| dots.replace(&x, r"\.").to_string())
        .collect();
    println!("extensions {}", exts.join("|"));

    //let pattern =
    //    format!(r"(.+)(?:[_-][Rr]?([12])?)?\.(?:{})$", exts.join("|"));

    let single = format!(r"(.+)\.(?:{})$", exts.join("|"));
    let paired = format!(r"(.+)[_-][Rr]?([12])?\.(?:{})$", exts.join("|"));
    println!("single re {}", single);
    println!("paired re {}", paired);

    let single_re = Regex::new(&single).unwrap();
    let paired_re = Regex::new(&paired).unwrap();

    for path in paths.into_iter().map(Path::new) {
        if let Some(file_name) = path.file_name() {
            let basename = file_name.to_string_lossy();
            println!("basename {}", basename);
            if let Some(cap) = paired_re.captures(&basename) {
                let sample_name = &cap[1];
                let for_rev = &cap[2];
                println!("sample_name {:?}", sample_name);
                println!("for_rev {:?}", for_rev);
            } else if let Some(cap) = single_re.captures(&basename) {
                let sample_name = &cap[1];
                println!("sample_name {:?}", sample_name);
            }
        }
    }

    let foo = vec!["foo".to_string()];
    Ok(foo)
}

// --------------------------------------------------
/// Returns the extension plus optional ".gz"
fn get_extension(path: &Path) -> Option<String> {
    let re = Regex::new(r"\.([^.]+(?:\.gz)?)$").unwrap();
    if let Some(basename) = path.file_name() {
        let basename = basename.to_string_lossy();
        if let Some(cap) = re.captures(&basename) {
            return Some(cap[1].to_string());
        }
    }
    None
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension() {
        assert_eq!(
            get_extension(Path::new("foo.fna")),
            Some("fna".to_string())
        );

        assert_eq!(
            get_extension(Path::new("foo.fasta.gz")),
            Some("fasta.gz".to_string())
        );

        assert_eq!(
            get_extension(Path::new("foo.fa.gz")),
            Some("fa.gz".to_string())
        );

        assert_eq!(
            get_extension(Path::new("foo.fasta")),
            Some("fasta".to_string())
        );

        assert_eq!(get_extension(Path::new("foo.fq")), Some("fq".to_string()));

        assert_eq!(get_extension(Path::new("foo")), None);
    }
}
