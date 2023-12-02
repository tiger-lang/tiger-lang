use std::{env, process};

const BUILD_CMD: &str = "build";
const SUBCOMMANDS: [&str; 1] = [BUILD_CMD];

mod build;

pub struct CommandOpts {
    subcommand: String,
    path_specs: Vec<String>,
}

impl CommandOpts {
    pub fn from_args() -> Result<Self, String> {
        let mut res = Self {
            subcommand: "".to_string(),
            path_specs: vec![],
        };

        let mut pos_args = vec![];

        let mut args = env::args();
        while let Some(arg) = args.next() {
            if arg.starts_with("-") {
                res.parse_flag(arg, &mut args)?;
            } else {
                pos_args.push(arg);
            }
        }

        if pos_args.len() < 2 {
            return Err(format!(
                "no subcommand specified: please provide one of the following: {}",
                SUBCOMMANDS.join(" ")
            ));
        }

        res.subcommand = pos_args[1].clone();
        res.path_specs = pos_args.into_iter().skip(2).collect();

        let mut valid = false;
        for valid_subcmd in SUBCOMMANDS {
            if valid_subcmd == res.subcommand {
                valid = true;
                break;
            }
        }

        if !valid {
            return Err(format!(
                "invalid subcommand '{}' specified. please provide one of the following: {}",
                res.subcommand,
                SUBCOMMANDS.join(" ")
            ));
        }

        Ok(res)
    }

    fn parse_flag(&mut self, _flag: String, _args: &mut env::Args) -> Result<(), String> {
        todo!()
    }
}

pub fn run(opts: &CommandOpts) {
    let res = match opts.subcommand.as_str() {
        BUILD_CMD => build::run(opts),
        _ => panic!("unknown subcommand {}", opts.subcommand),
    };

    if let Err(msg) = res {
        eprintln!("tiger {}: error: {}", opts.subcommand, msg);
        process::exit(1)
    }
}
