use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Preference {
    Require,
    Prefer,
    Ignore,
    Avoid,
    Prohibit,
}

pub struct TransportProperties {}

impl fmt::Display for Preference {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Preference::Require => f.write_str("Require"),
            Preference::Prefer => f.write_str("Prefer"),
            Preference::Ignore => f.write_str("Ignore"),
            Preference::Avoid => f.write_str("Avoid"),
            Preference::Prohibit => f.write_str("Prohibit"),
        }
    }
}
