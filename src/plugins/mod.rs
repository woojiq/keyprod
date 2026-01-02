pub mod event_echo;
pub mod event_history;

use anyhow::Context;
pub use event_echo::EventEcho;
pub use event_history::EventHistory;

#[derive(Debug, thiserror::Error)]
#[error("{err}")]
pub struct PluginInitError {
    err: String,
}

impl PluginInitError {
    pub fn new(err: &str) -> Self {
        Self { err: err.into() }
    }
}

#[async_trait::async_trait]
trait Plugin {
    fn describe(&self) -> &'static str;

    async fn run(&mut self, rx: tokio::sync::mpsc::Receiver<crate::Event>);
}

struct PluginFactory {}

impl PluginFactory {
    pub fn create_plugin(name: &str) -> Result<Box<dyn Plugin + Send>, PluginInitError> {
        match name {
            "echo" => Ok(Box::new(EventEcho::new())),
            "history" => Ok(Box::new(EventHistory::new())),
            _ => Err(PluginInitError::new(&format!(
                "plugin with name '{name}' doesn't exist"
            ))),
        }
    }

    pub fn create_and_init_plugins<T: AsRef<str>>(
        names: &[T],
    ) -> Result<Vec<Box<dyn Plugin + Send>>, PluginInitError> {
        let mut plugins = Vec::with_capacity(names.len());
        for name in names {
            let plugin = Self::create_plugin(name.as_ref())?;

            plugins.push(plugin);
        }

        Ok(plugins)
    }
}

pub fn spawn_plugins_runtime_thread(
    plugins_names: &[String],
) -> anyhow::Result<(
    std::thread::JoinHandle<()>,
    Vec<tokio::sync::mpsc::Sender<crate::Event>>,
)> {
    let plugins = PluginFactory::create_and_init_plugins(plugins_names)
        .context("Failed to initialize plugins")?;

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
                handles.push(tokio::spawn(async move { plugin.run(rx).await }));
            }

            for handle in handles {
                handle.await.expect("Failed to finish plugin.");
            }
        });
    });

    Ok((plugins_runtime, plugin_txs))
}
