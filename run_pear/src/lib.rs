extern crate clap;

use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;
use std::{fs::{self, File}};
//use regex::Regex;

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

fn classify(paths: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    //lookup = HashMap<String, Vec[String]>;
    //for path in paths {
    //    //if 
    //}

    let foo = vec!["foo".to_string()];
    Ok(foo)
}
