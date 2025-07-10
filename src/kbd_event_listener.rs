use std::{
    fs::File,
    io::Read,
    os::{fd::AsFd, unix::fs::OpenOptionsExt},
};

use nix::poll::{PollFd, PollFlags, PollTimeout};

use crate::kbd_event::{InputEvent, KbdEvent};

#[derive(Debug)]
pub struct DeviceFile {
    path: std::path::PathBuf,
}

impl DeviceFile {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

pub trait KbdEventListener {
    fn listen(&mut self);
}

pub struct LinuxKeyboardEventListener {
    sender: std::sync::mpsc::Sender<KbdEvent>,
    devices: Vec<DeviceFile>,
}

impl LinuxKeyboardEventListener {
    const MAX_EVENTS_PER_READ: usize = 16;

    pub fn new(sender: std::sync::mpsc::Sender<KbdEvent>, devices: Vec<DeviceFile>) -> Self {
        Self { sender, devices }
    }

    fn open_dev_files(&mut self) -> Vec<File> {
        self.devices
            .iter()
            .filter_map(|dev| {
                let opened_file = File::options()
                    .read(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(&dev.path);

                match opened_file {
                    Ok(file) => {
                        println!("Successfully opened {:?} for reading events.", dev.path);
                        Some(file)
                    }
                    Err(dev_err) => {
                        eprintln!(
                            "Failed to open device file {:?} to read: {}.",
                            dev.path, dev_err
                        );
                        None
                    }
                }
            })
            .collect()
    }

    fn create_poll_fds<'a>(&self, files: &'a [File]) -> Vec<PollFd<'a>> {
        files
            .iter()
            .map(|file| PollFd::new(file.as_fd(), PollFlags::POLLIN))
            .collect()
    }

    fn read_events_from_dev(&mut self, file: &mut File) -> Vec<KbdEvent> {
        let mut buf = [0; Self::MAX_EVENTS_PER_READ * size_of::<InputEvent>()];

        match file.read(&mut buf) {
            Ok(bytes_read) => {
                let all_events = unsafe { InputEvent::from_raw_bytes(&buf[..bytes_read]) };
                all_events
                    .into_iter()
                    .filter_map(|ev| ev.try_into().ok())
                    .collect()
            }
            Err(err) => {
                eprintln!("Error reading events from dev: {err}.");
                vec![]
            }
        }
    }

    fn send_events(
        &mut self,
        events: &[KbdEvent],
    ) -> Result<(), std::sync::mpsc::SendError<KbdEvent>> {
        for event in events {
            self.sender.send(*event)?;
        }

        Ok(())
    }
}

impl KbdEventListener for LinuxKeyboardEventListener {
    fn listen(&mut self) {
        let mut files = self.open_dev_files();

        println!(
            "Linux listener is ready to poll {} device files.",
            files.len()
        );

        'listen: loop {
            // TODO: Bruh, this looks so bad. Maybe there is a way to not recreated `pollfds`
            // from `files` every time?
            let mut pollfds = self.create_poll_fds(&files);
            assert_eq!(files.len(), pollfds.len());

            let _ = nix::poll::poll(&mut pollfds, PollTimeout::NONE).unwrap();

            // We need to collect to get drop of `pollfds` and make borrow checker happy.
            let need_poll = pollfds
                .into_iter()
                .enumerate()
                .filter_map(|(idx, pf)| {
                    pf.revents()
                        .and_then(|flags| flags.contains(PollFlags::POLLIN).then_some(idx))
                })
                .collect::<Vec<_>>();

            for idx in need_poll {
                // Looks like `nix::poll::poll` doesn't modify order of elements (the
                // example there uses the same approach) so it's safe to iterate together.
                let events = self.read_events_from_dev(&mut files[idx]);
                if let Err(err) = self.send_events(&events) {
                    eprintln!("Failed to send events via channel: {err}.");
                    break 'listen;
                }
            }
        }

        println!("Linux listener finished its loop.");
    }
}

pub fn get_all_kbd_devices() -> libudev::Result<Vec<DeviceFile>> {
    // TODO: react in live for new connected devices?
    let mut devices = vec![];

    let ctx = libudev::Context::new()?;
    let mut enumer = libudev::Enumerator::new(&ctx)?;
    for dev in enumer.scan_devices()? {
        if let (Some(id_val), Some(devname_val)) = (
            dev.property_value("ID_INPUT_KEYBOARD"),
            dev.property_value("DEVNAME"),
        ) {
            if id_val == "1" {
                let path = DeviceFile::new(std::path::PathBuf::from(devname_val));
                devices.push(path);
            }
        }
    }

    Ok(devices)
}
