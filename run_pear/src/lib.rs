extern crate clap;
extern crate regex;

use clap::{App, Arg};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::process::{Command, Stdio};
use std::{
    env,
    fs::{self, DirBuilder},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Config {
    query: Vec<String>,
    out_dir: PathBuf,
    num_concurrent_jobs: Option<u32>,
    num_halt: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum ReadDirection {
    Forward,
    Reverse,
}

type MyResult<T> = Result<T, Box<dyn Error>>;
type ReadPair = HashMap<ReadDirection, String>;
type ReadPairLookup = HashMap<String, ReadPair>;

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
        .arg(
            Arg::with_name("out_dir")
                .short("o")
                .long("out_dir")
                .value_name("DIR")
                .help("Output directory"),
        )
        .arg(
            Arg::with_name("num_concurrent_jobs")
                .short("n")
                .long("num_concurrent_jobs")
                .value_name("INT")
                .default_value("8")
                .help("Number of concurrent jobs for parallel"),
        )
        .arg(
            Arg::with_name("num_halt")
                .short("H")
                .long("num_halt")
                .value_name("INT")
                .default_value("1")
                .help("Halt after this many failing jobs"),
        )
        .get_matches();

    let out_dir = match matches.value_of("out_dir") {
        Some(x) => PathBuf::from(x),
        _ => {
            let cwd = env::current_dir()?;
            cwd.join(PathBuf::from("uproc-out"))
        }
    };

    let num_concurrent_jobs = matches
        .value_of("num_concurrent_jobs")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let num_halt = matches
        .value_of("num_halt")
        .and_then(|x| x.trim().parse::<u32>().ok());

    Ok(Config {
        query: matches.values_of_lossy("query").unwrap(),
        out_dir,
        num_concurrent_jobs,
        num_halt,
    })
}

// --------------------------------------------------
pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.query)?;

    if files.is_empty() {
        let msg = format!("No input files from query \"{:?}\"", &config.query);
        return Err(From::from(msg));
    }

    let pairs = classify(&files)?;

    println!("Found {} pair.", pairs.keys().len());

    let out_dir = &config.out_dir;
    if !out_dir.is_dir() {
        DirBuilder::new().recursive(true).create(&out_dir)?;
    }

    let jobs = make_jobs(&config, pairs)?;

    //println!("{:?}", jobs);

    run_jobs(
        &jobs,
        "Running pear",
        config.num_concurrent_jobs.unwrap_or(8),
        config.num_halt.unwrap_or(1),
    )?;

    println!("Done, see output in \"{:?}\"", &config.out_dir);

    Ok(())
}

// --------------------------------------------------
fn make_jobs(
    config: &Config,
    pairs: ReadPairLookup,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut jobs: Vec<String> = vec![];
    for (i, (sample, val)) in pairs.iter().enumerate() {
        println!("{:3}: {}", i + 1, sample);

        if let (Some(fwd), Some(rev)) = (
            val.get(&ReadDirection::Forward),
            val.get(&ReadDirection::Reverse),
        ) {
            let out_file = &config.out_dir.join(sample);
            jobs.push(format!(
                "pear -f {} -r {} -o {}",
                fwd,
                rev,
                out_file.display()
            ));
        }
    }

    Ok(jobs)
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
fn classify(paths: &[String]) -> Result<ReadPairLookup, Box<dyn Error>> {
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

    let pattern = format!(r"(.+)[_-][Rr]?([12])?\.(?:{})$", exts.join("|"));
    let re = Regex::new(&pattern).unwrap();

    let mut reads: ReadPairLookup = HashMap::new();
    for path in paths.into_iter().map(Path::new) {
        let path_str = path.to_str().expect("Convert path");

        if let Some(file_name) = path.file_name() {
            let basename = file_name.to_string_lossy();
            if let Some(cap) = re.captures(&basename) {
                let sample_name = &cap[1];
                let direction = if &cap[2] == "1" {
                    ReadDirection::Forward
                } else {
                    ReadDirection::Reverse
                };

                if !reads.contains_key(sample_name) {
                    let mut pair: ReadPair = HashMap::new();
                    pair.insert(direction, path_str.to_string());
                    reads.insert(sample_name.to_string(), pair);
                } else if let Some(pair) = reads.get_mut(sample_name) {
                    pair.insert(direction, path_str.to_string());
                }
            }
        }
    }

    //let mut bad = vec![];
    //for (key, val) in reads.iter() {
    //    if !val.contains_key(&ReadDirection::Forward)
    //        || !val.contains_key(&ReadDirection::Reverse)
    //    {
    //        bad.push(key.to_string());
    //    }
    //}

    let bad: Vec<String> = reads
        .iter()
        .filter_map(|(k, v)| {
            if !v.contains_key(&ReadDirection::Forward)
                || !v.contains_key(&ReadDirection::Reverse)
            {
                Some(k.to_string())
            } else {
                None
            }
        })
        .collect();

    for key in bad {
        reads.remove(&key);
    }

    if reads.is_empty() {
        Err(From::from("No pairs"))
    } else {
        Ok(reads)
    }
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
fn run_jobs(
    jobs: &[String],
    msg: &str,
    num_concurrent_jobs: u32,
    num_halt: u32,
) -> MyResult<()> {
    let num_jobs = jobs.len();

    if num_jobs > 0 {
        println!(
            "{} (# {} job{} @ {})",
            msg,
            num_jobs,
            if num_jobs == 1 { "" } else { "s" },
            num_concurrent_jobs,
        );

        let mut args: Vec<String> =
            vec!["-j".to_string(), num_concurrent_jobs.to_string()];

        if num_halt > 0 {
            args.push("--halt".to_string());
            args.push(format!("soon,fail={}", num_halt.to_string()));
        }

        let mut process = Command::new("parallel")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;

        {
            let stdin = process.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(jobs.join("\n").as_bytes())
                .expect("Failed to write to stdin");
        }

        let result = process.wait()?;
        if !result.success() {
            return Err(From::from("Failed to run jobs in parallel"));
        }
    }

    Ok(())
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

    #[test]
    fn test_classify() {
        assert!(classify(&["ERR1711926.fastq.gz".to_string()]).is_err());

        let res = classify(&[
            "/foo/bar/ERR1711926_1.fastq.gz".to_string(),
            "/foo/bar/ERR1711926_2.fastq.gz".to_string(),
            "/foo/bar/ERR1711927-R1.fastq.gz".to_string(),
            "/foo/bar/ERR1711927_R2.fastq.gz".to_string(),
            "/foo/bar/ERR1711928.fastq.gz".to_string(),
            "/foo/bar/ERR1711929_1.fastq.gz".to_string(),
        ]);
        assert!(res.is_ok());

        if let Ok(res) = res {
            assert!(res.len() == 2);
            assert!(res.contains_key("ERR1711926"));
            assert!(res.contains_key("ERR1711927"));
            assert!(!res.contains_key("ERR1711928"));
            assert!(!res.contains_key("ERR1711929"));

            if let Some(val) = res.get("ERR1711926") {
                assert!(val.contains_key(&ReadDirection::Forward));
                assert!(val.contains_key(&ReadDirection::Reverse));

                if let Some(fwd) = val.get(&ReadDirection::Forward) {
                    assert_eq!(fwd, &"/foo/bar/ERR1711926_1.fastq.gz");
                }
                if let Some(rev) = val.get(&ReadDirection::Reverse) {
                    assert_eq!(rev, &"/foo/bar/ERR1711926_2.fastq.gz");
                }
            }

            if let Some(val) = res.get("ERR1711927") {
                assert!(val.contains_key(&ReadDirection::Forward));
                assert!(val.contains_key(&ReadDirection::Reverse));

                if let Some(fwd) = val.get(&ReadDirection::Forward) {
                    assert_eq!(fwd, &"/foo/bar/ERR1711927-R1.fastq.gz");
                }
                if let Some(rev) = val.get(&ReadDirection::Reverse) {
                    assert_eq!(rev, &"/foo/bar/ERR1711927_R2.fastq.gz");
                }
            }
        }
    }
}
