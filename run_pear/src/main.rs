extern crate run_pear;
use std::process;

fn main() {
    let config = match run_pear::get_args() {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = run_pear::run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}
