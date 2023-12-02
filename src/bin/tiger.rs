use std::{
    io::{stderr, Write},
    process::exit,
};

use tiger_lang::cmd::{self, CommandOpts};

fn main() {
    match CommandOpts::from_args() {
        Ok(opts) => cmd::run(&opts),
        Err(msg) => {
            _ = stderr().write_fmt(format_args!("{}\n", msg));
            exit(1);
        }
    }
}
