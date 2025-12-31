use crate::time::CurrentLocalTime;

use super::KeyboardEventSubscriber;

pub struct EventHistory {
    logic: EventHistoryLogic<chrono::Local>,
}

struct EventHistoryLogic<T: CurrentLocalTime> {
    stats: KeycodeStatisticsSql,
    time: T,
}

struct KeycodeStatisticsSql {
    db_con: rusqlite::Connection,
}

impl EventHistory {
    const DB_NAME: &'static str = "history.db";

    pub fn new(state_dir: &std::path::Path) -> Self {
        let db_path = Self::get_db_path(state_dir);
        Self::precreate_dir(&db_path).unwrap();

        eprintln!("Trying to open history db at {db_path:?}");

        Self {
            logic: EventHistoryLogic::new(
                rusqlite::Connection::open(db_path).expect("Failed to open db"),
                chrono::Local {},
            ),
        }
    }

    fn get_db_path(state_dir: &std::path::Path) -> std::path::PathBuf {
        std::path::PathBuf::new()
            .join(state_dir)
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

impl KeyboardEventSubscriber for EventHistory {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent) {
        self.logic.register_keypress(event);
    }

    fn describe(&self) -> &'static str {
        "EventHistory"
    }
}

impl<T: CurrentLocalTime> EventHistoryLogic<T> {
    fn new(db_con: rusqlite::Connection, time: T) -> Self {
        Self {
            stats: KeycodeStatisticsSql::new(db_con),
            time,
        }
    }

    fn register_keypress(&mut self, event: crate::kbd_event::KbdEvent) {
        if !event.is_press() {
            return;
        }

        if !event.code.is_modifier() {
            self.stats.save_keypress(1, self.time.now().date_naive());
        }
    }
}

impl KeycodeStatisticsSql {
    pub fn new(db_con: rusqlite::Connection) -> Self {
        let mut obj = Self { db_con };

        obj.create_table();

        obj
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

    pub fn save_keypress(&mut self, cnt: u32, date: chrono::NaiveDate) {
        let update_query = "
            INSERT INTO Statistics(Date, KeypressCnt)
            VALUES(?1, ?2)
            ON CONFLICT(date)
            DO UPDATE SET KeypressCnt = KeypressCnt + ?2
         ";

        self.db_con
            .execute(update_query, [date.to_string(), cnt.to_string()])
            .unwrap();
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

    #[test]
    fn basic_test() {
        let mut time_mock = crate::time::MockCurrentLocalTime::new();
        time_mock
            .expect_now()
            .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
            .times(1);

        let mut history =
            EventHistoryLogic::new(rusqlite::Connection::open_in_memory().unwrap(), time_mock);

        history.register_keypress(KbdEvent {
            code: Keycode::KEY_1,
            state: KbdKeyState::Pressed,
        });

        history
            .stats
            .db_con
            .prepare("SELECT KeypressCnt FROM Statistics")
            .unwrap()
            .query_one([], |res| {
                assert_eq!(res.get::<_, u32>(0).unwrap(), 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn only_pressed_keyevents() {
        let mut time_mock = crate::time::MockCurrentLocalTime::new();
        time_mock
            .expect_now()
            .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
            .times(1);

        let mut history =
            EventHistoryLogic::new(rusqlite::Connection::open_in_memory().unwrap(), time_mock);

        history.register_keypress(KbdEvent {
            code: Keycode::KEY_1,
            state: KbdKeyState::Pressed,
        });
        history.register_keypress(KbdEvent {
            code: Keycode::KEY_1,
            state: KbdKeyState::Released,
        });

        history
            .stats
            .db_con
            .prepare("SELECT KeypressCnt FROM Statistics")
            .unwrap()
            .query_one([], |res| {
                assert_eq!(res.get::<_, u32>(0).unwrap(), 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn multiple_dates() {
        let mut time_mock = crate::time::MockCurrentLocalTime::new();
        time_mock
            .expect_now()
            .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
            .times(1);
        time_mock
            .expect_now()
            .return_const(chrono::Local.with_ymd_and_hms(2000, 1, 2, 0, 0, 0).unwrap())
            .times(1);

        let mut history =
            EventHistoryLogic::new(rusqlite::Connection::open_in_memory().unwrap(), time_mock);

        history.register_keypress(KbdEvent {
            code: Keycode::KEY_1,
            state: KbdKeyState::Pressed,
        });
        history.register_keypress(KbdEvent {
            code: Keycode::KEY_1,
            state: KbdKeyState::Pressed,
        });

        let mut stmt = history
            .stats
            .db_con
            .prepare("SELECT Date, KeypressCnt FROM Statistics")
            .unwrap();
        let mut res = stmt.query([]).unwrap();

        let res0 = res.next().unwrap().unwrap();
        assert_eq!(res0.get::<_, String>(0).unwrap(), "2000-01-01".to_string());
        assert_eq!(res0.get::<_, u32>(1).unwrap(), 1);

        let res1 = res.next().unwrap().unwrap();
        assert_eq!(res1.get::<_, String>(0).unwrap(), "2000-01-02".to_string());
        assert_eq!(res1.get::<_, u32>(1).unwrap(), 1);
    }
}
