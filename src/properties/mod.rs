use enum_map::{enum_map, EnumMap};
use lazy_static::lazy_static;

use std::fmt;

pub use selection::SelectionProperty;
pub use security::SecurityParameters;

mod selection;
mod security;

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
        SelectionProperty::Multipath => Preference::Prefer,
        SelectionProperty::RetransmitNotify => Preference::Ignore,
        SelectionProperty::SoftErrorNotify => Preference::Ignore,
    };
}

/// The preference an application has for a specific property.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Preference {
    /// Select only protocols/paths providing the property, fail otherwise.
    Require,

    /// Prefer protocol/paths providing the property, proceed otherwise.
    Prefer,

    /// No preference.
    Ignore,

    /// Prefer protocols/paths not providing the property, proceed otherwise.
    Avoid,

    /// Select only protocols/paths not providing the property, fail otherwise.
    Prohibit,
}

/// The direction a Connection must support.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    /// The connection must support sending and receiving data.
    Bidirectional,

    /// The connection must support sending data.
    Sender,

    /// The connection must support receiving data.
    Receiver,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportProperties {
    select_props: EnumMap<SelectionProperty, Preference>,
    direction: Direction,
}

impl Default for TransportProperties {
    fn default() -> Self {
        TransportProperties {
            select_props: *DEFAULT,
            direction: Default::default(),
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

impl Default for Direction {
    fn default() -> Self {
        Direction::Bidirectional
    }
}

impl TransportProperties {
    /// Specify the [Preference](enum.Preference.html) for the given
    /// [SelectionProperty](enum.SelectionProperty.html).
    pub fn add(&mut self, prop: SelectionProperty, pref: Preference) -> &mut Self {
        self.select_props[prop] = pref;
        self
    }

    /// Specify the given [SelectionProperty](enum.SelectionProperty.html) is required.
    ///
    /// This is equivalent to `add(prop, Preference::Require)`.
    pub fn require(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Require)
    }

    /// Specify the given [SelectionProperty](enum.SelectionProperty.html) is prefered.
    ///
    /// This is equivalent to `add(prop, Preference::Prefer)`.
    pub fn prefer(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Prefer)
    }

    /// Specify there is no [Preference](enum.Preference.html) for the given
    /// [SelectionProperty](enum.SelectionProperty.html).
    ///
    /// This is equivalent to `add(prop, Preference::Ignore)`.
    pub fn ignore(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Ignore)
    }

    /// Specify the given [SelectionProperty](enum.SelectionProperty.html) should be avoided.
    ///
    /// This is equivalent to `add(prop, Preference::Avoid)`.
    pub fn avoid(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Avoid)
    }

    /// Specify the given [SelectionProperty](enum.SelectionProperty.html) is prohibited.
    ///
    /// This is equivalent to `add(prop, Preference::Prohibit)`.
    pub fn prohibit(&mut self, prop: SelectionProperty) -> &mut Self {
        self.add(prop, Preference::Prohibit)
    }

    /// Specify the given [SelectionProperty](enum.SelectionProperty.html) should have the default
    /// [Preference](enum.Preference.html).
    pub fn default_prop(&mut self, prop: SelectionProperty) -> &mut Self {
        let pref = DEFAULT[prop];
        self.add(prop, pref)
    }

    /// Specify whether an application wants to use the Connection for sending and/or reding data.
    ///
    /// If `Sender` or `Receiver` is given, and the unidirectional connections are not supported by
    /// the transport system, `Bidirectional` will be used as a fallback.
    pub fn direction(&mut self, dir: Direction) -> &mut Self {
        self.direction = dir;
        self
    }
}
