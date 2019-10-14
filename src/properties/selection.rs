use enum_map::Enum;

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
