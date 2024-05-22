use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceType {
    /// A host address
    A = 1,

    /// An authoritative name server
    NS,

    /// A mail destination (OBSOLETE - use [`MX`][ResourceType::MX])
    MD,

    /// A mail forwarder (OBSOLETE - use [`MX`][ResourceType::MX])
    MF,

    /// The canonical name for an alias
    CNAME,

    /// Marks the start of a zone of authority
    SOA,

    /// A mailbox domain name (EXPERIMENTAL)
    MB,

    /// A mail group member (EXPERIMENTAL)
    MG,

    /// A mail rename domain name (EXPERIMENTAL)
    MR,

    /// A null RR (EXPERIMENTAL)
    NULL,

    /// A well known service description
    WKS,

    /// A domain name pointer
    PTR,

    /// Host information
    HINFO,

    /// Mailbox or mail list information
    MINFO,

    /// Mail exchange
    MX,

    /// Text strings
    TXT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuestionType {
    /// A host address
    A = 1,

    /// An authoritative name server
    NS,

    /// A mail destination (OBSOLETE - use [`MX`][QuestionType::MX])
    MD,

    /// A mail forwarder (OBSOLETE - use [`MX`][QuestionType::MX])
    MF,

    /// The canonical name for an alias
    CNAME,

    /// Marks the start of a zone of authority
    SOA,

    /// A mailbox domain name (EXPERIMENTAL)
    MB,

    /// A mail group member (EXPERIMENTAL)
    MG,

    /// A mail rename domain name (EXPERIMENTAL)
    MR,

    /// A null RR (EXPERIMENTAL)
    NULL,

    /// A well known service description
    WKS,

    /// A domain name pointer
    PTR,

    /// Host information
    HINFO,

    /// Mailbox or mail list information
    MINFO,

    /// Mail exchange
    MX,

    /// Text strings
    TXT,

    /// A request for a transfer of an entire zone
    AXFR = 252,

    /// A request for mailbox-related records (MB, MG or MR)
    MAILB,

    /// A request for mail agent RRs (OBSOLETE - see [`MX`][QuestionType::MX])
    MAILA,

    /// A request for all records
    ALL,
}

/// No type has been registered with code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnregisteredType(u16);

impl Display for UnregisteredType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format!("no type has been registered with code '{}'", self.0).fmt(f)
    }
}

impl Error for UnregisteredType {}

impl TryFrom<u16> for ResourceType {
    type Error = UnregisteredType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use ResourceType::*;
        Ok(match value {
            1 => A,
            2 => NS,
            3 => MD,
            4 => MF,
            5 => CNAME,
            6 => SOA,
            7 => MB,
            8 => MG,
            9 => MR,
            10 => NULL,
            11 => WKS,
            12 => PTR,
            13 => HINFO,
            14 => MINFO,
            15 => MX,
            16 => TXT,
            code => return Err(UnregisteredType(code)),
        })
    }
}

impl TryFrom<u16> for QuestionType {
    type Error = UnregisteredType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use QuestionType::*;
        Ok(match value {
            1 => A,
            2 => NS,
            3 => MD,
            4 => MF,
            5 => CNAME,
            6 => SOA,
            7 => MB,
            8 => MG,
            9 => MR,
            10 => NULL,
            11 => WKS,
            12 => PTR,
            13 => HINFO,
            14 => MINFO,
            15 => MX,
            16 => TXT,
            252 => AXFR,
            253 => MAILB,
            254 => MAILA,
            255 => ALL,
            code => return Err(UnregisteredType(code)),
        })
    }
}

/// No class has been registered with code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnregisteredClass(u16);

impl Display for UnregisteredClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format!("no class has been registered with code '{}'", self.0).fmt(f)
    }
}

impl Error for UnregisteredClass {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceClass {
    /// The Internet
    IN = 1,

    /// The CSNET class (OBSOLETE - used only for examples in some obsolete RFCs)
    CS,

    /// The CHAOS class
    CH,

    /// Hesiod [Dyer 87]
    HS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuestionClass {
    /// The Internet
    IN = 1,

    /// The CSNET class (OBSOLETE - used only for examples in some obsolete RFCs)
    CS,

    /// The CHAOS class
    CH,

    /// Hesiod [Dyer 87]
    HS,

    /// A request for all records
    Any = 255,
}

impl TryFrom<u16> for ResourceClass {
    type Error = UnregisteredClass;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use ResourceClass::*;
        Ok(match value {
            1 => IN,
            2 => CS,
            3 => CH,
            4 => HS,
            code => return Err(UnregisteredClass(code)),
        })
    }
}

impl TryFrom<u16> for QuestionClass {
    type Error = UnregisteredClass;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use QuestionClass::*;
        Ok(match value {
            1 => IN,
            2 => CS,
            3 => CH,
            4 => HS,
            255 => Any,
            code => return Err(UnregisteredClass(code)),
        })
    }
}
