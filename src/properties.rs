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
    Receiver
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Enum)]
pub enum SelectionProperty {
    /// Reliable data transfer.
    ///
    /// This property specifies whether the application needs to use a transport protocol that
    /// ensures that all data is received on the other side without corruption. This also entails
    /// being notified when a Connection is closed or aborted.
    ///
    /// # Default
    ///
    /// By default, this is `Require`. Changes to default values are **not** considered a breaking
    /// change.
    Reliability,

    /// Configure per-message reliability
    ///
    /// This property specifies whether an application considers it useful to indicate its
    /// reliability requirements on a per-Message basis. This property applies to Connections and
    /// Connection Groups.
    ///
    /// # Default
    ///
    /// By default, this is `Prefer`. Changes to default values are **not** considered a breaking
    /// change.
    PerMsgReliability,

    /// Preservation of message boundaries
    ///
    /// This property specifies whether the application needs or prefers to  use a transport
    /// protocol that preserves message boundaries.
    ///
    /// # Default
    ///
    /// By default, this is `Ignore`. Changes to default values are **not** considered a breaking
    /// change.
    PreserveMsgBoundaries,

    /// Preservation of data ordering
    ///
    /// This property specifies whether the application wishes to use a transport protocol that can
    /// ensure that data is received by the application on the other end in the same order as it was
    /// sent.
    ///
    /// # Default
    ///
    /// By default, this is `Require`. Changes to default values are **not** considered a breaking
    /// change.
    PreserveOrder,

    /// Use 0-RTT session establishment with an idempotent message
    ///
    /// This property specifies whether an application would like to supply a Message to the
    /// transport protocol before Connection establishment, which will then be reliably transferred
    /// to the other side before or during Connection establishment, potentially multiple times
    /// (i.e., multiple copies of the message data may be passed to the Remote Endpoint).
    ///
    /// # Default
    ///
    /// By default, this is `Ignore`. Changes to default values are **not** considered a breaking
    /// change.
    ZeroRttMsg,

    /// Multistream connections in group
    ///
    /// This property specifies that the application would prefer multiple Connections within a
    /// Connection Group to be provided by streams of a single underlying transport connection where
    /// possible.
    ///
    /// # Default
    ///
    /// By default, this is `Prefer`. Changes to default values are **not** considered a breaking
    /// change.
    Multistreaming,

    /// Full checksum coverage on sending
    ///
    /// This property specifies whether the application desires protection against corruption for
    /// all data transmitted on this Connection. Disabling this property may enable to control
    /// checksum coverage later.
    ///
    /// # Default
    ///
    /// By default, this is `Require`. Changes to default values are **not** considered a breaking
    /// change.
    PerMsgChecksumLenSend,

    /// Full checksum coverage on receiving
    ///
    /// This property specifies whether the application desires protection against corruption for
    /// all data received on this Connection.
    ///
    /// # Default
    ///
    /// By default, this is `Require`. Changes to default values are **not** considered a breaking
    /// change.
    PerMsgChecksumLenRecv,

    /// Congestion control
    ///
    /// This property specifies whether the application would like the Connection to be congestion
    /// controlled or not. Note that if a Connection is not congestion controlled, an application
    /// using such a Connection should itself perform congestion control in accordance with
    /// [RFC2914](https://tools.ietf.org/html/rfc2914).  Also note that reliability is usually
    /// combined with congestion control in protocol implementations, rendering "reliable but not
    /// congestion controlled" a request that is unlikely to succeed.
    ///
    /// # Default
    ///
    /// By default, this is `Require`. Changes to default values are **not** considered a breaking
    /// change.
    CongestionControl,

    /// Interface instance or type
    ///
    /// TODO
    Interface,

    /// Provisioning domain instance or type
    ///
    /// TODO
    Pvd,

    /// Parallel use of multiple paths
    ///
    /// This property specifies whether an application considers it useful to transfer data across
    /// multiple paths between the same end hosts. Generally, in most cases, this will improve
    /// performance (e.g., achieve greater throughput).  One possible side-effect is increased
    /// jitter, which may be problematic for delay-sensitive applications.
    ///
    /// # Default
    ///
    /// By default, this is `Prefer`. Changes to default values are **not** considered a breaking
    /// change.
    Multipath,

    /// Notification of excessive retransmissions
    ///
    /// This property specifies whether an application considers it useful to be informed in case
    /// sent data was retransmitted more often than a certain threshold.
    ///
    /// # Default
    ///
    /// By default, this is `Ignore`. Changes to default values are **not** considered a breaking
    /// change.
    RetransmitNotify,

    /// Notification of ICMP soft error message arrival
    ///
    /// This property specifies whether an application considers it useful to be informed when an
    /// ICMP error message arrives that does not force termination of a connection. When set to
    /// true, received ICMP errors will be available as SoftErrors. Note that even if a protocol
    /// supporting this property is selected, not all ICMP errors will necessarily be delivered, so
    /// applications cannot rely on receiving them.
    ///
    /// # Default
    ///
    /// By default, this is `Ignore`. Changes to default values are **not** considered a breaking
    /// change.
    SoftErrorNotify,
}

pub struct TransportProperties {
    select_props: EnumMap<SelectionProperty, Preference>,
    direction: Direction
}

impl Default for TransportProperties {
    fn default() -> Self {
        TransportProperties {
            select_props: DEFAULT.clone(),
            direction: Default::default()
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
    pub fn default(&mut self, prop: SelectionProperty) -> &mut Self {
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
