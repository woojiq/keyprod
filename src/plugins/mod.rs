pub mod echo;
pub mod history;

use anyhow::Context;
pub use echo::EventEcho;
pub use history::PluginHistory;

use crate::plugins::{echo::EventEchoFactory, history::PluginHistoryFactory};

pub static PLUGINS: [&dyn PluginFactory; 2] = [&EventEchoFactory, &PluginHistoryFactory];

#[derive(Debug, thiserror::Error)]
#[error("{plugin_name}: {err}")]
pub struct PluginInitError {
    plugin_name: String,
    err: Box<dyn std::error::Error + Send + Sync>,
}

impl PluginInitError {
    pub fn new(plugin: &str, err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            plugin_name: plugin.into(),
            err,
        }
    }
}

#[async_trait::async_trait]
pub trait Plugin: Send {
    fn name(&self) -> &'static str;

    async fn run(&mut self, rx: tokio::sync::mpsc::Receiver<crate::Event>);
}

pub trait PluginConfig {
    fn name(&self) -> &'static str;

    fn try_init_plugin(
        self: Box<Self>,
    ) -> Result<Box<dyn Plugin>, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait PluginFactory: Sync {
    fn cli_name(&self) -> &'static str;

    fn help(&self) -> String;

    fn parse_args(
        &self,
        parser: &mut lexopt::Parser,
    ) -> Result<Box<dyn PluginConfig>, lexopt::Error>;
}

pub fn parse_plugin_from_args(
    args: Vec<std::ffi::OsString>,
) -> Result<Box<dyn PluginConfig>, lexopt::Error> {
    let mut parser = lexopt::Parser::from_iter(args);

    let bin_name = parser
        .bin_name()
        .ok_or_else(|| lexopt::Error::MissingValue {
            option: Some("<plugin_name>".to_string()),
        })?
        .to_string();

    for plugin in PLUGINS {
        if bin_name == plugin.cli_name() {
            return plugin.parse_args(&mut parser);
        }
    }

    Err(lexopt::Error::UnexpectedValue {
        option: "<plugin_name>".to_string(),
        value: bin_name.into(),
    })
}

fn init_plugins_from_configs(
    configs: Vec<Box<dyn PluginConfig>>,
) -> Result<Vec<Box<dyn Plugin>>, PluginInitError> {
    let mut plugins = Vec::with_capacity(configs.len());

    for config in configs {
        let name = config.name();

        match config.try_init_plugin() {
            Ok(plugin) => plugins.push(plugin),
            Err(err) => return Err(PluginInitError::new(name, err)),
        }
    }

    Ok(plugins)
}

pub fn spawn_plugins_runtime_thread(
    plugins_configs: Vec<Box<dyn PluginConfig>>,
) -> anyhow::Result<(
    std::thread::JoinHandle<()>,
    Vec<tokio::sync::mpsc::Sender<crate::Event>>,
)> {
    let plugins =
        init_plugins_from_configs(plugins_configs).context("Failed to initialize plugins")?;

    let (mut plugin_txs, mut plugin_rxs) = (
        Vec::with_capacity(plugins.len()),
        Vec::with_capacity(plugins.len()),
    );
    for _ in 0..plugins.len() {
        let (tx, rx) = tokio::sync::mpsc::channel::<crate::Event>(256);
        plugin_txs.push(tx);
        plugin_rxs.push(rx);
    }

    let plugins_runtime = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to spawn tokio runtime.");

        runtime.block_on(async {
            let mut handles = Vec::with_capacity(plugins.len());

            for (mut plugin, rx) in plugins.into_iter().zip(plugin_rxs) {
                let name = plugin.name();
                handles.push((name, tokio::spawn(async move { plugin.run(rx).await })));
            }

            for (name, handle) in handles {
                if let Err(err) = handle.await {
                    log::error!("The plugin \"{name}\" failed to gracefully finish: {err}");
                }
            }
        });
    });

    Ok((plugins_runtime, plugin_txs))
}
