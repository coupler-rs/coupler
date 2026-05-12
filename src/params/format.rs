use std::fmt::{self, Display};
use std::str::FromStr;

pub trait Format<T> {
    fn parse(&self, text: &str) -> Option<T>;
    fn display(&self, value: T, write: impl fmt::Write) -> Result<(), fmt::Error>;
}

pub struct DefaultFormat;

impl<T: FromStr + Display> Format<T> for DefaultFormat {
    fn parse(&self, text: &str) -> Option<T> {
        T::from_str(text).ok()
    }

    fn display(&self, value: T, mut write: impl fmt::Write) -> Result<(), fmt::Error> {
        write!(write, "{}", value)
    }
}
