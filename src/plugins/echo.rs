use super::Plugin;

const PLUGIN_NAME: &str = "Echo";

pub struct EventEcho {
    writer: Box<dyn std::io::Write + Send>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Default, Clone)]
pub struct EventEchoConfig {
    writer: EventEchoWriter,
}

#[derive(Default, Clone)]
pub enum EventEchoWriter {
    #[default]
    Stdout,
    Stderr,
    File(std::path::PathBuf),
}

pub struct EventEchoFactory;

impl EventEcho {
    fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self { writer }
    }

    pub fn init(value: EventEchoConfig) -> Result<Self> {
        match value.writer {
            EventEchoWriter::Stdout => {
                let stdout = std::io::stdout();
                Ok(Self::new(Box::new(stdout)))
            }
            EventEchoWriter::Stderr => {
                let stderr = Box::new(std::io::stderr());
                Ok(Self::new(Box::new(stderr)))
            }
            EventEchoWriter::File(path_buf) => {
                let file = std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open(path_buf)?;
                Ok(Self::new(Box::new(file)))
            }
        }
    }

    fn keyevent_cb(&mut self, event: crate::kbd_event::KbdEvent) {
        if event.is_press() || event.is_release() {
            if let Err(err) = writeln!(
                self.writer,
                "{}({}) was {}",
                event.code,
                event.code as u16,
                if event.is_press() {
                    "Pressed"
                } else {
                    "Released"
                },
            ) {
                log::error!("Failed to log keypress: {err}");
            }
        }
    }
}

#[async_trait::async_trait]
impl Plugin for EventEcho {
    fn name(&self) -> &'static str {
        PLUGIN_NAME
    }

    async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<crate::Event>) {
        while let Some(event) = rx.recv().await {
            match event {
                crate::Event::KeyEvent(kbd_ev) => self.keyevent_cb(kbd_ev),
                crate::Event::PluginStop => break,
            }
        }
    }
}

impl super::PluginConfig for EventEchoConfig {
    fn name(&self) -> &'static str {
        PLUGIN_NAME
    }

    fn try_init_plugin(
        self: Box<Self>,
    ) -> Result<Box<dyn Plugin>, Box<dyn std::error::Error + Send + Sync>> {
        match EventEcho::init(*self) {
            Ok(pl) => Ok(Box::new(pl)),
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl super::PluginFactory for EventEchoFactory {
    fn cli_name(&self) -> &'static str {
        "echo"
    }

    fn help(&self) -> String {
        "\
Log keyboard events as is.
By default if the plugin is enabled it prints to stdout.
Options:
    --stdout|stderr
        Print events to stdout|stderr.
    --file <file-name>
        Saves events to the file.
"
        .to_string()
    }

    fn parse_args(
        &self,
        parser: &mut lexopt::Parser,
    ) -> Result<Box<dyn super::PluginConfig>, lexopt::Error> {
        use lexopt::prelude::*;

        let mut config = EventEchoConfig::default();

        while let Some(arg) = parser.next()? {
            match arg {
                Long("stdout") => config.writer = EventEchoWriter::Stdout,
                Long("stderr") => config.writer = EventEchoWriter::Stderr,
                Long("file") => config.writer = EventEchoWriter::File(parser.value()?.into()),
                _ => return Err(arg.unexpected()),
            }
        }

        Ok(Box::new(config))
    }
}
