const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

pub struct Args {
    pub subscribers: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            subscribers: vec!["echo".into()],
        }
    }
}

pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut args = Args::default();
    let mut subscribers = vec![];

    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("plugins") => {
                if let Ok(vals) = parser.values() {
                    for val in vals {
                        subscribers.push(val.parse()?);
                    }
                }
                // Ignore error, it simply means 0 plugins were explicitly specified.
            }
            Long("help") => {
                print_help();
                std::process::exit(0);
            }
            Long("version") => {
                print_version();
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    if !subscribers.is_empty() {
        args.subscribers = subscribers;
    }

    Ok(args)
}

pub fn print_version() {
    println!("{VERSION}");
}

pub fn print_help() {
    println!(
        "\
{VERSION}
Track keyboard productivity.

Usage:
    {PKG_NAME} [options]

Options:
    --plugins ...
        List of features to enable
        Available plugins: echo, history
    --help
        Prints help information
    --version
        Prints version
    "
    );
}
