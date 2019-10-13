use enum_map::{Enum, EnumMap, enum_map};
use lazy_static::lazy_static;

use std::fmt;

lazy_static! {
    static ref DEFAULT: EnumMap<SelectionProperty, Preference> = enum_map! {
                SelectionProperty::Reliability => Preference::Require,
                SelectionProperty::PreserveMsgBoundaries => Preference::Prefer,
                SelectionProperty::PerMsgReliability => Preference::Ignore,
                SelectionProperty::PreserveOrder => Preference::Require,
                SelectionProperty::ZeroRttMsg => Preference::Ignore,
                SelectionProperty::Multistreaming => Preference::Prefer,
                SelectionProperty::PerMsgChecksumLenSend => Preference::Require,
                SelectionProperty::PerMsgChecksumLenRecv => Preference::Require,
                SelectionProperty::CongestionControl => Preference::Require,
                SelectionProperty::Interface => Preference::Ignore,
                SelectionProperty::Pvd => Preference::Ignore,
                SelectionProperty::Multipath => Preference::Prefer,
                //SelectionProperty::Direction(Direction::Bidirectional) => Preference::Require,
                SelectionProperty::RetransmitNotify => Preference::Ignore,
                SelectionProperty::SoftErrorNotify => Preference::Ignore,
            };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Preference {
    Require,
    Prefer,
    Ignore,
    Avoid,
    Prohibit,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    Bidirectional,
    Sender,
    Receiver
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Enum)]
pub enum SelectionProperty {
    Reliability,
    PreserveMsgBoundaries,
    PerMsgReliability,
    PreserveOrder,
    ZeroRttMsg,
    Multistreaming,
    PerMsgChecksumLenSend,
    PerMsgChecksumLenRecv,
    CongestionControl,
    Interface,
    Pvd,
    Multipath,
    //Direction(Direction),
    RetransmitNotify,
    SoftErrorNotify,
}

pub struct TransportProperties {
    map: EnumMap<SelectionProperty, Preference>
}

impl Default for TransportProperties {
    fn default() -> Self {
        TransportProperties {
            map: DEFAULT.clone()
        }
    }
}

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

impl TransportProperties {
    pub fn add(&mut self, prop: SelectionProperty, pref: Preference) -> &mut Self {
        self.map[prop] = pref;
        self
    }

    pub fn require(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Require)
    }

    pub fn prefer(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Prefer)
    }

    pub fn ignore(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Ignore)
    }

    pub fn avoid(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Avoid)
    }

    pub fn prohibit(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Prohibit)
    }

    pub fn default(&mut self, prop: SelectionProperty) -> &mut Self {
        let pref = DEFAULT[prop];
        self.add(prop, pref)
    }
}
