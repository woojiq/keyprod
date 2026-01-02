use anyhow::{Context, anyhow};
use keyprod::{
    kbd_event_listener::spawn_kbd_event_listener_thread, plugins::spawn_plugins_runtime_thread,
    publisher::spawn_publisher_thread,
};

extern "C" fn signal_handler(_: libc::c_int) {
    keyprod::kbd_event_listener::stop_listening();
}

fn get_signal_handler() -> libc::sighandler_t {
    signal_handler as extern "C" fn(libc::c_int) as *mut libc::c_void as libc::sighandler_t
}

fn main() -> anyhow::Result<()> {
    let args = keyprod::args::parse_args().context("Failed to parse command line args")?;

    unsafe {
        libc::signal(libc::SIGINT, get_signal_handler());
        libc::signal(libc::SIGTERM, get_signal_handler());
    }

    let (kbd_tx, kbd_rx) = std::sync::mpsc::channel::<keyprod::kbd_event::KbdEvent>();

    let listener_thread = spawn_kbd_event_listener_thread(kbd_tx)
        .context("Failed to spawn kbd event listener thread")?;

    let (plugins_runtime_thread, txs_to_plugins) = spawn_plugins_runtime_thread(&args.plugins)
        .context("Failed to spawn plugins runtime thread")?;

    let publisher_thread = spawn_publisher_thread(kbd_rx, txs_to_plugins)
        .context("Failed to spawn publisher thread")?;

    // Graceful shutdown sequence:
    // SIGINT, SIGTERM is received =>
    // CONTINUE_LISTEN is set =>
    // kbd_event_listener stops =>
    // kbd_tx is dropped =>
    // kbd_rx returns an error =>
    // publisher stops =>
    // all plugins stop

    listener_thread
        .join()
        .map_err(|err| anyhow!("Error joining thread (Listener) {err:?}"))?;
    publisher_thread
        .join()
        .map_err(|err| anyhow!("Error joining thread (Publisher) {err:?}"))?;
    plugins_runtime_thread
        .join()
        .map_err(|err| anyhow!("Error joining thread (Plugins) {err:?}"))?;

    Ok(())
}
