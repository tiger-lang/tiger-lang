use std::{fs, io};

use super::CommandOpts;

fn run_internal(opts: &CommandOpts) -> io::Result<()> {
    for path in &opts.path_specs {
        for entry in fs::read_dir(&path)? {
            println!("{}", entry?.path().to_string_lossy())
        }
    }

    Ok(())
}

pub fn run(opts: &CommandOpts) -> Result<(), String> {
    run_internal(opts).map_err(|e| e.to_string())
}
