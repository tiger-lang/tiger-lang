use std::{
    fs::{self, File},
    io,
};

use crate::tokenizer;

use super::CommandOpts;

fn run_internal(opts: &CommandOpts) -> io::Result<()> {
    for path in &opts.path_specs {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path().to_string_lossy().to_string();
            println!("{}", path);

            let file = File::open(entry.path())?;
            let tok = tokenizer::TokenStream::new(file, Some(path));

            for token in tok {
                match token {
                    Ok(t) => println!("\t{:?}", t),
                    Err(err) => {
                        println!("\t{}", err.message);
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn run(opts: &CommandOpts) -> Result<(), String> {
    run_internal(opts).map_err(|e| e.to_string())
}
