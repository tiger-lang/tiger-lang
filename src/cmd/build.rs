use std::{
    fs::{self, File},
    io,
};

use crate::parser;

use super::CommandOpts;

fn run_internal(opts: &CommandOpts) -> io::Result<()> {
    for path in &opts.path_specs {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path().to_string_lossy().to_string();
            println!("{}", path);

            let file = File::open(entry.path())?;
            let module_name = entry.file_name().to_string_lossy().to_string();
            let mut parser = parser::Parser::new(module_name);
	    let res = parser.add_source(file, Some(path.clone()));
	    if let Err(e) = res {
		println!("Failed to process {}: {}", path, e);
	    }
        }
    }

    Ok(())
}

pub fn run(opts: &CommandOpts) -> Result<(), String> {
    run_internal(opts).map_err(|e| e.to_string())
}
