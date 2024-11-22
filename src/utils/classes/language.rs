use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Copy)]
pub enum Language {
    English,
    German,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
