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
    p_value: Option<f32>,
    min_overlap: Option<u32>,
    max_assembly_length: Option<u32>,
    min_assembly_length: Option<u32>,
    min_trim_length: Option<u32>,
    quality_threshold: Option<u32>,
    max_uncalled_base: Option<f32>,
    test_method: Option<u32>,
    empirical_freqs: Option<bool>,
    score_method: Option<u32>,
    phred_base: Option<u32>,
    memory: Option<String>,
    cap: Option<u32>,
    threads: Option<u32>,
    nbase: Option<bool>,
    keep_original: Option<bool>,
    stitch: Option<bool>,
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
                .short("Q")
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
            Arg::with_name("p_value")
                .short("p")
                .long("p_value")
                .value_name("FLOAT")
                .help("P-value"),
        )
        .arg(
            Arg::with_name("min_overlap")
                .short("v")
                .long("min_overlap")
                .value_name("INT")
                .help("Minimum overlap"),
        )
        .arg(
            Arg::with_name("max_assembly_length")
                .short("m")
                .long("max_assembly_length")
                .value_name("INT")
                .help("Max assembly length"),
        )
        .arg(
            Arg::with_name("min_assembly_length")
                .short("n")
                .long("min_assembly_length")
                .value_name("INT")
                .help("Max assembly length"),
        )
        .arg(
            Arg::with_name("min_trim_length")
                .short("t")
                .long("min_trim_length")
                .value_name("INT")
                .help("Minimum assembly length"),
        )
        .arg(
            Arg::with_name("quality_threshold")
                .short("q")
                .long("quality_threshold")
                .value_name("INT")
                .help("Quality threshold"),
        )
        .arg(
            Arg::with_name("max_uncalled_base")
                .short("u")
                .long("max_uncalled_base")
                .value_name("FLOAT")
                .help("Max proportion of uncalled bases"),
        )
        .arg(
            Arg::with_name("test_method")
                .short("g")
                .long("test_method")
                .value_name("INT")
                .help("Type  of  statistical  test"),
        )
        .arg(
            Arg::with_name("empirical_freqs")
                .short("e")
                .long("empirical_freqs")
                .help("Disable  empirical base frequencies"),
        )
        .arg(
            Arg::with_name("score_method")
                .short("s")
                .long("score_method")
                .value_name("INT")
                .help("Scoring method"),
        )
        .arg(
            Arg::with_name("phred_base")
                .short("b")
                .long("phred_base")
                .value_name("INT")
                .help("Base PHRED quality score"),
        )
        .arg(
            Arg::with_name("memory")
                .short("y")
                .long("memory")
                .value_name("STR")
                .help("Amount of memory to be used"),
        )
        .arg(
            Arg::with_name("cap")
                .short("c")
                .long("cap")
                .value_name("INT")
                .help("Upper bound for the resulting quality score"),
        )
        .arg(
            Arg::with_name("threads")
                .short("j")
                .long("threads")
                .value_name("INT")
                .help("Number of threads to use"),
        )
        .arg(
            Arg::with_name("nbase")
                .short("z")
                .long("nbase")
                .help("Nbase"),
        )
        .arg(
            Arg::with_name("keep_original")
                .short("k")
                .long("keep_original")
                .help("Keep original"),
        )
        .arg(
            Arg::with_name("stitch")
                .short("i")
                .long("stitch")
                .help("concatenate reads"),
        )
        .arg(
            Arg::with_name("num_concurrent_jobs")
                .short("J")
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
            cwd.join(PathBuf::from("pear-out"))
        }
    };

    let p_value = matches
        .value_of("p_value")
        .and_then(|x| x.trim().parse::<f32>().ok());

    let min_overlap = matches
        .value_of("min_overlap")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let max_assembly_length = matches
        .value_of("max_assembly_length")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let min_assembly_length = matches
        .value_of("min_assembly_length")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let min_trim_length = matches
        .value_of("min_trim_length")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let quality_threshold = matches
        .value_of("quality_threshold")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let max_uncalled_base = matches
        .value_of("max_uncalled_base")
        .and_then(|x| x.trim().parse::<f32>().ok());

    let test_method = matches
        .value_of("test_method")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let empirical_freqs = Some(matches.is_present("empirical_freqs"));

    let score_method = matches
        .value_of("score_method")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let phred_base = matches
        .value_of("phred_base")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let memory = if let Some(memory) = matches.value_of("memory") {
        Some(memory.to_string())
    } else {
        None
    };

    let cap = matches
        .value_of("cap")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let threads = matches
        .value_of("threads")
        .and_then(|x| x.trim().parse::<u32>().ok());

    let nbase = Some(matches.is_present("nbase"));

    let keep_original = Some(matches.is_present("keep_original"));

    let stitch = Some(matches.is_present("stitch"));

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
        p_value,
        min_overlap,
        max_assembly_length,
        min_assembly_length,
        min_trim_length,
        quality_threshold,
        max_uncalled_base,
        test_method,
        empirical_freqs,
        score_method,
        phred_base,
        memory,
        cap,
        threads,
        nbase,
        keep_original,
        stitch,
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

    println!("Processing {} pair.", pairs.keys().len());

    let out_dir = &config.out_dir;
    if !out_dir.is_dir() {
        DirBuilder::new().recursive(true).create(&out_dir)?;
    }

    let jobs = make_jobs(&config, pairs)?;

    run_jobs(
        &jobs,
        "Running pear",
        config.num_concurrent_jobs.unwrap_or(8),
        config.num_halt.unwrap_or(1),
    )?;

    println!("Done, see output in \"{}\"", &config.out_dir.display());

    Ok(())
}

// --------------------------------------------------
fn make_jobs(
    config: &Config,
    pairs: ReadPairLookup,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut args: Vec<String> = vec![];
    if let Some(p_value) = config.p_value {
        args.push(format!("-p {}", p_value));
    }

    if let Some(min_overlap) = config.min_overlap {
        args.push(format!("-v {}", min_overlap));
    }

    if let Some(max_assembly_length) = config.max_assembly_length {
        args.push(format!("-m {}", max_assembly_length));
    }

    if let Some(min_assembly_length) = config.min_assembly_length {
        args.push(format!("-n {}", min_assembly_length));
    }

    if let Some(min_trim_length) = config.min_trim_length {
        args.push(format!("-t {}", min_trim_length));
    }

    if let Some(quality_threshold) = config.quality_threshold {
        args.push(format!("-q {}", quality_threshold));
    }

    if let Some(max_uncalled_base) = config.max_uncalled_base {
        args.push(format!("-u {}", max_uncalled_base));
    }

    if let Some(test_method) = config.test_method {
        args.push(format!("-g {}", test_method));
    }

    if let Some(empirical_freqs) = config.empirical_freqs {
        if empirical_freqs {
            args.push("-e".to_string());
        }
    }

    if let Some(score_method) = config.score_method {
        args.push(format!("-s {}", score_method));
    }

    if let Some(phred_base) = config.phred_base {
        args.push(format!("-b {}", phred_base));
    }

    if let Some(memory) = &config.memory {
        args.push(format!("-y {}", memory));
    }

    if let Some(cap) = config.cap {
        args.push(format!("-c {}", cap));
    }

    if let Some(threads) = config.threads {
        args.push(format!("-j {}", threads));
    }

    if let Some(nbase) = config.nbase {
        if nbase {
            args.push("-z".to_string());
        }
    }

    if let Some(keep_original) = config.keep_original {
        if keep_original {
            args.push("-k".to_string());
        }
    }

    if let Some(stitch) = config.stitch {
        if stitch {
            args.push("-i".to_string());
        }
    }

    let mut jobs: Vec<String> = vec![];
    for (i, (sample, val)) in pairs.iter().enumerate() {
        println!("{:3}: {}", i + 1, sample);

        if let (Some(fwd), Some(rev)) = (
            val.get(&ReadDirection::Forward),
            val.get(&ReadDirection::Reverse),
        ) {
            let out_file = &config.out_dir.join(sample);
            jobs.push(format!(
                "pear -f {} -r {} -o {} {}",
                fwd,
                rev,
                out_file.display(),
                args.join(" "),
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
    let paths = paths.iter().map(Path::new);
    let mut exts: Vec<String> =
        paths.clone().map(get_extension).filter_map(|x| x).collect();
    exts.dedup();

    let dots = Regex::new(r"\.").unwrap();
    let exts: Vec<String> = exts
        .into_iter()
        .map(|x| dots.replace(&x, r"\.").to_string())
        .collect();

    let pattern = format!(r"(.+)[_-][Rr]?([12])?\.(?:{})$", exts.join("|"));
    let re = Regex::new(&pattern).unwrap();

    let mut reads: ReadPairLookup = HashMap::new();
    for path in paths.map(Path::new) {
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
