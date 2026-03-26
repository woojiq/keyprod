use crate::time::CurrentLocalTime;

use super::Plugin;

const PLUGIN_NAME: &str = "History";

pub struct PluginHistory<T: CurrentLocalTime> {
    unsaved_events: u64,
    dump_interval: Option<std::time::Duration>,

    stats_db: KeycodeStatisticsSql,
    time: T,
}

struct KeycodeStatisticsSql {
    db_con: rusqlite::Connection,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Db(#[from] rusqlite::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct PluginHistoryConfig {
    db_path: std::path::PathBuf,
    dump_interval: u64,
}

pub struct PluginHistoryFactory;

impl<T: CurrentLocalTime> PluginHistory<T> {
    fn new(
        stats_db: KeycodeStatisticsSql,
        dump_interval: Option<std::time::Duration>,
        time: T,
    ) -> Self {
        Self {
            unsaved_events: 0,
            dump_interval,
            stats_db,
            time,
        }
    }

    pub fn init(value: PluginHistoryConfig, time: T) -> Result<Self> {
        let sql = KeycodeStatisticsSql::new(&value.db_path)?;

        let interval = if value.dump_interval == 0 {
            None
        } else {
            Some(std::time::Duration::from_secs(value.dump_interval))
        };

        Ok(Self::new(sql, interval, time))
    }

    fn save_keypress_in_cache(&mut self, event: crate::kbd_event::KbdEvent) {
        if !event.is_press() {
            return;
        }

        if !event.code.is_modifier() {
            self.unsaved_events += 1;
        }
    }

    fn sync_db_with_cache(&mut self) -> Result<()> {
        if self.unsaved_events != 0 {
            self.stats_db
                .save_keypress(self.unsaved_events, self.time.now().date_naive())
                .inspect_err(|err| log::error!("Failed to save keypresses to the db: {err}"))?;

            log::debug!("Saved {} keypresses into db.", self.unsaved_events);
            self.unsaved_events = 0;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: CurrentLocalTime> Plugin for PluginHistory<T> {
    // TODO: think whether we need `name` function in each trait.
    fn name(&self) -> &'static str {
        PLUGIN_NAME
    }

    async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<crate::Event>) {
        let mut interval = self.dump_interval.map(tokio::time::interval);

        loop {
            tokio::select! {
                event_res = rx.recv() => {
                    let Some(event) = event_res else {
                        break;
                    };

                    match event {
                        crate::Event::KeyEvent(kbd_ev) => {
                            self.save_keypress_in_cache(kbd_ev);
                            if interval.is_none() {
                                let _ = self.sync_db_with_cache();
                            }
                        },
                        crate::Event::PluginStop => {
                            let _ = self.sync_db_with_cache();
                        },
                    }
                },
                _ = async {
                    if let Some(interval) = interval.as_mut() {
                        interval.tick().await
                    } else {
                        std::future::pending().await
                    }
                } => {
                    let _ = self.sync_db_with_cache();
                }
            };
        }
    }
}

impl KeycodeStatisticsSql {
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let mut obj = Self {
            db_con: rusqlite::Connection::open_in_memory()?,
        };

        obj.setup()?;

        Ok(obj)
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        log::info!("Trying to open db {:?}", path.as_ref());
        let mut obj = Self {
            db_con: rusqlite::Connection::open(path.as_ref())?,
        };

        obj.setup()?;

        Ok(obj)
    }

    fn setup(&mut self) -> Result<()> {
        self.enable_wal()?;
        self.create_table()?;

        Ok(())
    }

    fn enable_wal(&mut self) -> Result<()> {
        // Without WAL we can't read db while the service is running in the background:
        // Parse error: database is locked (5)
        self.db_con.pragma_update(None, "journal_mode", "WAL")?;
        Ok(())
    }

    fn create_table(&mut self) -> Result<()> {
        const CMD: &str = "
            CREATE TABLE IF NOT EXISTS Statistics (
                Date TEXT PRIMARY KEY,
                KeypressCnt INT NOT NULL DEFAULT 0
            )
        ";

        self.db_con.execute(CMD, [])?;
        Ok(())
    }

    pub fn save_keypress(&mut self, cnt: u64, date: chrono::NaiveDate) -> Result<()> {
        let update_query = "
            INSERT INTO Statistics(Date, KeypressCnt)
            VALUES(?1, ?2)
            ON CONFLICT(date)
            DO UPDATE SET KeypressCnt = KeypressCnt + ?2
         ";

        self.db_con
            .execute(update_query, [date.to_string(), cnt.to_string()])?;

        Ok(())
    }
}

impl Default for PluginHistoryConfig {
    fn default() -> Self {
        Self {
            db_path: std::path::PathBuf::from(crate::STATE_DIR).join("history.db"),
            dump_interval: 5,
        }
    }
}

impl super::PluginConfig for PluginHistoryConfig {
    fn name(&self) -> &'static str {
        PLUGIN_NAME
    }

    fn try_init_plugin(
        self: Box<Self>,
    ) -> Result<Box<dyn Plugin>, Box<dyn std::error::Error + Send + Sync>> {
        match PluginHistory::init(*self, chrono::Local {}) {
            Ok(pl) => Ok(Box::new(pl)),
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl super::PluginFactory for PluginHistoryFactory {
    fn cli_name(&self) -> &'static str {
        "history"
    }

    fn help(&self) -> String {
        "\
Stores the number of keyboard presses for each day in a database.
Only regular keys are counted, modifiers (like Shift) are ignored.
Useful to monitor your performance on the computer (if you run this program as a service).
Options:
    --db-path <path>
        Absolute path to the db where to store the number of daily keyboard presses.

        [default: /var/lib/keyprod/history.db]

    --interval <sec>
        Interval between saving events to the database. Increasing the value can reduce the load
        on the system, as keyboard events will be summed up locally for longer and synced with the
        database less often. If the value is 0, changes to the database will be saved with each
        keystroke.

        [default: 5]
"
        .to_string()
    }

    fn parse_args(
        &self,
        parser: &mut lexopt::Parser,
    ) -> Result<Box<dyn super::PluginConfig>, lexopt::Error> {
        use lexopt::prelude::*;

        let mut config = PluginHistoryConfig::default();

        while let Some(arg) = parser.next()? {
            match arg {
                Long("db-path") => config.db_path = parser.value()?.into(),
                Long("interval") => config.dump_interval = parser.value()?.parse()?,
                _ => return Err(arg.unexpected()),
            }
        }

        Ok(Box::new(config))
    }
}

#[cfg(test)]
mod tests {
    // use chrono::TimeZone;

    // use crate::{
    //     kbd_event::{KbdEvent, KbdKeyState},
    //     keycode::Keycode,
    // };

    // use super::*;

    // #[test]
    // fn basic_test() {
    //     let mut time_mock = crate::time::MockCurrentLocalTime::new();
    //     time_mock
    //         .expect_now()
    //         .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
    //         .times(1);

    //     let mut history =
    //         EventHistoryLogic::new(rusqlite::Connection::open_in_memory().unwrap(), time_mock);

    //     history.register_keypress(KbdEvent {
    //         code: Keycode::KEY_1,
    //         state: KbdKeyState::Pressed,
    //     });

    //     history
    //         .stats
    //         .db_con
    //         .prepare("SELECT KeypressCnt FROM Statistics")
    //         .unwrap()
    //         .query_one([], |res| {
    //             assert_eq!(res.get::<_, u32>(0).unwrap(), 1);
    //             Ok(())
    //         })
    //         .unwrap();
    // }

    // #[test]
    // fn only_pressed_keyevents() {
    //     let mut time_mock = crate::time::MockCurrentLocalTime::new();
    //     time_mock
    //         .expect_now()
    //         .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
    //         .times(1);

    //     let mut history =
    //         EventHistoryLogic::new(rusqlite::Connection::open_in_memory().unwrap(), time_mock);

    //     history.register_keypress(KbdEvent {
    //         code: Keycode::KEY_1,
    //         state: KbdKeyState::Pressed,
    //     });
    //     history.register_keypress(KbdEvent {
    //         code: Keycode::KEY_1,
    //         state: KbdKeyState::Released,
    //     });

    //     history
    //         .stats
    //         .db_con
    //         .prepare("SELECT KeypressCnt FROM Statistics")
    //         .unwrap()
    //         .query_one([], |res| {
    //             assert_eq!(res.get::<_, u32>(0).unwrap(), 1);
    //             Ok(())
    //         })
    //         .unwrap();
    // }

    // #[test]
    // fn multiple_dates() {
    //     let mut time_mock = crate::time::MockCurrentLocalTime::new();
    //     time_mock
    //         .expect_now()
    //         .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
    //         .times(1);
    //     time_mock
    //         .expect_now()
    //         .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 2, 0, 0, 0).unwrap())
    //         .times(1);

    //     let mut history =
    //         EventHistoryLogic::new(rusqlite::Connection::open_in_memory().unwrap(), time_mock);

    //     history.register_keypress(KbdEvent {
    //         code: Keycode::KEY_1,
    //         state: KbdKeyState::Pressed,
    //     });
    //     history.register_keypress(KbdEvent {
    //         code: Keycode::KEY_1,
    //         state: KbdKeyState::Pressed,
    //     });

    //     let mut stmt = history
    //         .stats
    //         .db_con
    //         .prepare("SELECT Date, KeypressCnt FROM Statistics")
    //         .unwrap();
    //     let mut res = stmt.query([]).unwrap();

    //     let res0 = res.next().unwrap().unwrap();
    //     assert_eq!(res0.get::<_, String>(0).unwrap(), "2000-01-01".to_string());
    //     assert_eq!(res0.get::<_, u32>(1).unwrap(), 1);

    //     let res1 = res.next().unwrap().unwrap();
    //     assert_eq!(res1.get::<_, String>(0).unwrap(), "2000-01-02".to_string());
    //     assert_eq!(res1.get::<_, u32>(1).unwrap(), 1);
    // }
}
