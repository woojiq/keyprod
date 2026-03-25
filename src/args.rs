use crate::{
    PKG_NAME, VERSION,
    plugins::{PLUGINS, PluginConfig},
};
use lexopt::prelude::*;

pub struct Args {
    pub plugins: Vec<Box<dyn PluginConfig>>,
    pub general: GeneralArgs,
}

pub struct GeneralArgs {}

impl Args {
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            general: GeneralArgs {},
        }
    }

    pub fn parse() -> Result<Self, lexopt::Error> {
        let mut args = Self::new();

        let mut parser = lexopt::Parser::from_env();

        // FIXME: sudo target/debug/keyprod
        // Error: missing argument for option '<plugin_name>'
        args.parse_app_args(&mut parser)?;
        args.parse_plugins_args(&mut parser)?;

        Ok(args)
    }

    fn parse_app_args(&mut self, parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
        #[allow(clippy::never_loop)]
        while let Some(arg) = parser.next()? {
            match arg {
                Long("help") => {
                    print_help();
                    std::process::exit(0);
                }
                Long("version") => {
                    print_version();
                    std::process::exit(0);
                }
                Long("plugin") => break,
                _ => return Err(arg.unexpected()),
            }
        }

        Ok(())
    }

    fn parse_plugins_args(&mut self, parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
        let mut args = vec![];

        let mut raw_args = parser.try_raw_args().ok_or(lexopt::Error::MissingValue {
            option: Some("plugin".into()),
        })?;

        let mut expect_plugin = true;
        while expect_plugin {
            expect_plugin = false;

            for arg in &mut raw_args {
                if arg == "--plugin" {
                    expect_plugin = true;
                    break;
                } else {
                    args.push(arg);
                }
            }

            self.plugins
                .push(crate::plugins::parse_plugin_from_args(std::mem::take(
                    &mut args,
                ))?);
        }

        Ok(())
    }
}

pub fn print_version() {
    println!("{VERSION}");
}

pub fn print_help() {
    let mut help_msg = format!(
        "\
{VERSION}
Track keyboard productivity.

By default, logs with level \"info\" or higher are printed. To control the log level, use the
environment variable \"RUST_LOG\". For example, \"RUST_LOG=warn\" to reduce the verbosity to only
warnings and error messages; or \"RUST_LOG=debug\" to include debug messages.

Usage:
    {PKG_NAME} [options]

Options:
    --plugin <plugin-name> [plugin-options]...
        Enable plugin and configure it. Multiple plugins can be enabled.
    --help
        Print help information.
    --version
        Print version.

Available plugins:

"
    );

    for plugin in PLUGINS {
        help_msg += &format!("* Plugin: {}\n{}\n", plugin.cli_name(), plugin.help());
    }

    println!("{help_msg}");
}
