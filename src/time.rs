#[cfg_attr(test, mockall::automock)]
pub trait CurrentLocalTime {
    fn now(&self) -> chrono::DateTime<chrono::Local>;
}

impl CurrentLocalTime for chrono::Local {
    fn now(&self) -> chrono::DateTime<chrono::Local> {
        chrono::Local::now()
    }
}
