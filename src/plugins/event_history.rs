use crate::time::CurrentLocalTime;

use super::Plugin;

pub struct EventHistory {
    logic: EventHistoryLogic<chrono::Local>,
}

struct EventHistoryLogic<T: CurrentLocalTime> {
    unsaved_events: u64,

    stats_db: KeycodeStatisticsSql,
    time: T,
}

struct KeycodeStatisticsSql {
    db_con: rusqlite::Connection,
}

impl EventHistory {
    const DB_NAME: &'static str = "history.db";

    pub fn new() -> Self {
        let db_path = Self::get_db_path();
        Self::precreate_dir(&db_path).expect("Failed to precreate dir");

        eprintln!("Trying to open history db at {db_path:?}");

        Self {
            logic: EventHistoryLogic::new(
                rusqlite::Connection::open(db_path).expect("Failed to open db"),
                chrono::Local {},
            ),
        }
    }

    fn get_db_path() -> std::path::PathBuf {
        std::path::PathBuf::new()
            .join(crate::STATE_DIR)
            .join(Self::DB_NAME)
    }

    fn precreate_dir(path: &std::path::Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            if !dir.exists() {
                return std::fs::create_dir_all(dir);
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Plugin for EventHistory {
    fn describe(&self) -> &'static str {
        "EventHistory"
    }

    async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<crate::Event>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                event_res = rx.recv() => {
                    let Some(event) = event_res else {
                        break;
                    };

                    match event {
                        crate::Event::KeyEvent(kbd_ev) => self.logic.save_keypress_inmemory(kbd_ev),
                        crate::Event::PluginStop => self.logic.save_keypresses_in_db(),
                    }
                },
                _ = interval.tick() => {
                    self.logic.save_keypresses_in_db();
                }
            };
        }
    }
}

impl<T: CurrentLocalTime> EventHistoryLogic<T> {
    fn new(db_con: rusqlite::Connection, time: T) -> Self {
        Self {
            unsaved_events: 0,

            stats_db: KeycodeStatisticsSql::new(db_con),
            time,
        }
    }

    fn save_keypress_inmemory(&mut self, event: crate::kbd_event::KbdEvent) {
        if !event.is_press() {
            return;
        }

        if !event.code.is_modifier() {
            self.unsaved_events += 1;
        }
    }

    fn save_keypresses_in_db(&mut self) {
        if self.unsaved_events != 0 {
            self.stats_db
                .save_keypress(self.unsaved_events, self.time.now().date_naive());

            eprintln!("Saved {} keypresses into db.", self.unsaved_events);
            self.unsaved_events = 0;
        }
    }
}

impl KeycodeStatisticsSql {
    pub fn new(db_con: rusqlite::Connection) -> Self {
        let mut obj = Self { db_con };

        obj.enable_wal();
        obj.create_table();

        obj
    }

    fn enable_wal(&mut self) {
        // Without WAL we can't read db while the service is running in the background:
        // Parse error: database is locked (5)
        self.db_con
            .pragma_update(None, "journal_mode", "WAL")
            .unwrap();
    }

    fn create_table(&mut self) {
        const CMD: &str = "
            CREATE TABLE IF NOT EXISTS Statistics (
                Date TEXT PRIMARY KEY,
                KeypressCnt INT NOT NULL DEFAULT 0
            )
        ";
        self.db_con.execute(CMD, []).unwrap();
    }

    pub fn save_keypress(&mut self, cnt: u64, date: chrono::NaiveDate) {
        let update_query = "
            INSERT INTO Statistics(Date, KeypressCnt)
            VALUES(?1, ?2)
            ON CONFLICT(date)
            DO UPDATE SET KeypressCnt = KeypressCnt + ?2
         ";

        let _ = self
            .db_con
            .execute(update_query, [date.to_string(), cnt.to_string()]);
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::{
        kbd_event::{KbdEvent, KbdKeyState},
        keycode::Keycode,
    };

    use super::*;

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
