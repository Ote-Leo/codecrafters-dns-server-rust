//! The [`answer`], [`authority`], and [`additional`] sections all share the same format: a
//! variable number of [`resource records`], where the number of records is specified in the
//! corresponding count field in the [`header`]. Each resource record has the following format:
//!
//! ```txt
//!                                     1  1  1  1  1  1
//!       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                                               |
//!     /                                               /
//!     /                      NAME                     /
//!     |                                               |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                      TYPE                     |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                     CLASS                     |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                      TTL                      |
//!     |                                               |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                   RDLENGTH                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
//!     /                     RDATA                     /
//!     /                                               /
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! ```
//!
//! [`answer`]: super::Message::answers
//! [`authority`]: super::Message::authorities
//! [`additional`]: super::Message::additionals
//! [`resource records`]: ResourceRecord
//! [`header`]: super::header::Header

// TODO: use a safer parsing (i.e. check for buffer boundaries)

use bytes::{Buf, BufMut};

use super::{
    parse_character_string, parse_label, CharacterString, Label, LabelError, ResourceClass,
    ResourceType, UnregisteredClass, UnregisteredType,
};
use std::{
    error::Error,
    fmt::{self, Display},
    net::Ipv4Addr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRecord {
    /// A domain name to which this resource record pertains.
    pub name: Label,

    /// This field specifies the class of the [`data`][ResourceRecord::data].
    pub class: ResourceClass,

    /// Specifies the time interval (in seconds) that the resource record may be cached before it
    /// should be discarded. Zero values are interpreted to mean that the RR can only be used for
    /// the transaction in progress, and should not be cached.
    pub time_to_live: u32,

    pub data: ResourceData,
}

impl ResourceRecord {
    pub fn typ(&self) -> ResourceType {
        self.data.typ()
    }
}

impl From<ResourceRecord> for Vec<u8> {
    fn from(value: ResourceRecord) -> Self {
        let mut buf = vec![];
        let data = value.data;

        buf.extend::<Vec<_>>(value.name.into());
        buf.put_u16(data.typ() as u16);
        buf.put_u16(value.class as u16);
        buf.put_u32(value.time_to_live);

        let data: Vec<u8> = data.into();
        buf.put_u16(data.len() as u16);
        buf.extend(data);

        buf
    }
}

// TODO: fix documentation here
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceData {
    /// (A) Internet Address
    ///
    /// Hosts that have multiple Internet addresses will have multiple A records.
    ///
    /// A records cause no additional section processing. The RDATA section of an A line in a
    /// master file is an Internet address expressed as four decimal number separated by dots
    /// wihtout any imbedded spaces (e.g., "10.2.0.52" or "192.0.5.6").
    Address(Ipv4Addr),

    /// (NS) A domain name which specifies a host which should be authoritative for the specified
    /// class and domain.
    ///
    /// NS records cause both the usual additional section processing to locate a type A record,
    /// and, when used in a referral, a special search of the zone in which they reside for glue
    /// information.
    ///
    ///
    /// The NS RR states that the named host should be expected to have a zone starting at owner
    /// name of the specified class. Note that the class may not indicate the protocol family which
    /// should be used to communicate with the host, although it is typically a strong hint. For
    /// example, hosts which are name servers for either Internet (IN) or Hesiod (HS) class
    /// information are normally queried using IN class protocols.
    NameServer(Label),

    /// (MD) A domain name which specifies a host which has a mail agent for the domain which should be
    /// able to deliver mail for the domain.
    ///
    /// MD records cause additional section processing which looks up an A type records
    /// corresponding to MADNAME.
    ///
    /// MD is obsolete. See the definition of MX and RFC-974 for details of the new scheme. The
    /// recommended policy for dealing with MD RRs found in a master file is to reject them, or to
    /// convert them to MX RRs with a preference of 0.
    MailDevice(Label),

    /// (MF) A domain name which specifies a host which has a mail agent for the domain which will
    /// accept mail for forwarding to the domain.
    ///
    /// MF recrods cause additinoal section processing which looks up an A type record
    /// corresponding to MADNAME.
    ///
    /// MF is obsolete. See the definition of MX and RFC-974 for details ofw the new scheme. The
    /// recommended policy for dealing with MD RRs found in a master file is to reject them, or to
    /// convert them to MX RRs with a preference of 10.
    MailForward(Label),

    /// (CNAME) A domain name which specifies the canonical or primary name for the owner. The owner name
    /// is an alias.
    CanonicalName(Label),

    /// (SOA) SOA records cause no additional section processing.
    ///
    /// All times are in units of seconds.
    ///
    /// Most of these fields are pertinent only for name server maintenance operations. However,
    /// MINIMUM is used in all query operations that retrieve RRs from a zone. Whenever a RR is
    /// sent in a response to a query, the TTL field is set to the maximum of the TL field from the
    /// RR and the MINIMUM field in the appropriate SOA. Thus MINIMUM is a lower bbound on the TTL
    /// field for all RRs in a zone. Note that this use of MINIMUM should occur when the RRs are
    /// copied into the response and not when the zone is loaded from a master file or via a zone
    /// transfer. The reason for this provision is to allow future dynamic update facilities to
    /// change the SOA RR with known semantics.
    SOA {
        /// The domain name of the name server that was the original or primary source of data for
        /// this zone.
        name: Label,

        /// The domain name which specifies the mailbox of the person responsible for this zone.
        mail: Label,

        /// Version number of the original copy of the zone. Zone transfers preserve this value.
        /// This value wraps and should be compared using sequence space arithmetic.
        serial: u32,

        /// Time interval before the zone should be refereshed.
        refresh: u32,

        /// Time interval that should elapse before a failed refresh should be retried.
        retry: u32,

        /// Time value that specifies the upper limit on the time interval that can elapse before
        /// the zone is no longer authoritative.
        expire: u32,

        /// Minimum TTL field that should be exported with any RR from this zone.
        minimum: u32,
    },

    /// (MB) A domain name which specifies a host which has the specified mailbox.
    ///
    /// MB records cause additinoal section processing which looks up an A type RRs corresponding
    /// to MADNAME.
    MailBox(Label),

    /// (MG) A domain name which specifies a mailbox which is a memeber of a mailgroup specified by
    /// the domain name.
    ///
    /// MG records cause no additional section processing.
    MailGroup(Label),

    /// (MR) A domain name which specifies a mailbox which is the proper rename of the specified
    /// mailbox.
    MailRename(Label),

    /// (NULL) Anything at all may be in the RDATA field so long as it is 65535 octets or less.
    ///
    /// NULL records cause no additional section processing. NULL RRs are not allowed in master
    /// files. NULLs are used as placeholders in some experimental extensions of the DNS.
    Null(Vec<u8>),

    // (WKS) The WKS record is used to describe the well known services supported by a particular
    // protocol on a particular internet address.  The PROTOCOL field specifies an IP protocol
    // number, and the bit map has one bit per port of the specified protocol.  The first bit
    // corresponds to port 0, the second to port 1, etc.  If the bit map does not include a bit for
    // a protocol of interest, that bit is assumed zero.  The appropriate values and mnemonics for
    // ports and protocols are specified in RFC-1010.
    //
    // For example, if PROTOCOL=TCP (6), the 26th bit corresponds to TCP port
    // 25 (SMTP).  If this bit is set, a SMTP server should be listening on TCP port 25; if zero,
    //    SMTP service is not supported on the specified address.
    //
    // The purpose of WKS RRs is to provide availability information for servers for TCP and UDP.
    // If a server supports both TCP and UDP, or has multiple Internet addresses, then multiple WKS
    // RRs are used.
    //
    // WKS RRs cause no additional section processing.
    //
    // In master files, both ports and protocols are expressed using mnemonics or decimal numbers.
    WKS {
        address: Ipv4Addr,
        /// IP protocol number
        protocol: u8,
        /// TODO: implement bit map
        bit_map: (),
    },

    /// (PTR) A domain name which points to some locaiton in the domain name space.
    ///
    /// PTR records cause no addtional section processing. These RRs are used in special domain to
    /// point to some other location in the domain space. These records are simple data, and don't
    /// imply any special processing similar to that performed by CNAME, which idetifies aliases.
    /// See the description of the IN-ADD.ARPA domain for an example.
    Ptr(Label),

    /// (HINFO) Used to acquire general information about a host. The main use is for protocols such as FTP
    /// that can use special procedures when talking between machines or operating systems of the
    /// same type.
    ///
    /// Standard values for CPU and OS can be found in [RFC1010].
    ///
    /// [RFC1010]: <https://datatracker.ietf.org/doc/html/rfc1010>
    HostInfo {
        cpu: CharacterString,
        os: CharacterString,
    },

    /// (MINFO) MINFO records cause no additional section processing. Although these records can be
    /// associated with a simple mailbox, they are usually used with a mailing list.
    MailInfo {
        /// A domain name which specifies a mailbox which is reponsible for the mailing list or
        /// mailbox. If this domain name names the root, the owner ofthe MINFO RR is responsible
        /// for itself. Note that many existing mailing lists use a mailbox X-request for the
        /// RMAILBX field of mailing list X, e.g., Msgroup-request for Msgroup. This field provides
        /// a more general mechanism.
        mailbox: Label,

        /// A domain name which specifies a mailbox which is to  receive error messages related to
        /// the mailing list or mailbox specified by the owner of the MINFO RR (similar to the
        /// ERRORS-TS: field which has been proposed). If this domain name names the root, errors
        /// should be returned to the sender of the message.
        error_mailbox: Label,
    },

    /// (MX) MX records cause type A additional section processing for the host specified by
    /// EXCHANGE. The use of MX RRs is explained in detail in RFC-974.
    MailExchange {
        /// The preference given to this RR among others at the same owner. Lower values are
        /// preferred.
        preference: u16,

        /// A domain name which specifies a host willing to act as a mail exchange for the owner
        /// name.
        exchange: Label,
    },

    /// (TXT) One or more character string
    ///
    /// TXT RRs are usedto hold descriptive text. The semantics of the text depends on the domain
    /// where it is found.
    Text(Vec<CharacterString>),
}

impl ResourceData {
    pub fn typ(&self) -> ResourceType {
        use ResourceType::*;
        match self {
            ResourceData::Address(_) => A,
            ResourceData::NameServer(_) => NS,
            ResourceData::MailDevice(_) => MD,
            ResourceData::MailForward(_) => MF,
            ResourceData::CanonicalName(_) => CNAME,
            ResourceData::SOA { .. } => SOA,
            ResourceData::MailBox(_) => MB,
            ResourceData::MailGroup(_) => MG,
            ResourceData::MailRename(_) => MR,
            ResourceData::Null(_) => NULL,
            ResourceData::WKS { .. } => WKS,
            ResourceData::Ptr(_) => PTR,
            ResourceData::HostInfo { .. } => HINFO,
            ResourceData::MailInfo { .. } => MINFO,
            ResourceData::MailExchange { .. } => MX,
            ResourceData::Text(_) => TXT,
        }
    }
}

impl From<ResourceData> for Vec<u8> {
    fn from(value: ResourceData) -> Self {
        use ResourceData::*;
        let mut buf = vec![];

        match value {
            HostInfo { cpu, os } => {
                buf.extend::<Vec<_>>(cpu.into());
                buf.extend::<Vec<_>>(os.into());
            }

            CanonicalName(name) | MailDevice(name) | MailRename(name) | MailForward(name)
            | MailBox(name) | MailGroup(name) | NameServer(name) | Ptr(name) => {
                buf.extend::<Vec<_>>(name.into())
            }

            MailInfo {
                mailbox,
                error_mailbox,
            } => {
                buf.extend::<Vec<_>>(mailbox.into());
                buf.extend::<Vec<_>>(error_mailbox.into());
            }

            MailExchange {
                preference,
                exchange,
            } => {
                buf.put_u16(preference);
                buf.extend::<Vec<_>>(exchange.into());
            }

            Null(bytes) => buf.extend(bytes),

            SOA {
                name,
                mail,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            } => {
                buf.extend::<Vec<_>>(name.into());
                buf.extend::<Vec<_>>(mail.into());
                buf.put_u32(serial);
                buf.put_u32(refresh);
                buf.put_u32(retry);
                buf.put_u32(expire);
                buf.put_u32(minimum);
            }

            Text(text) => {
                for word in text {
                    buf.extend::<Vec<_>>(word.into());
                }
            }

            Address(ip) => buf.put_u32(ip.into()),

            WKS { .. } => todo!("implement the WKS serialization"),
        }

        buf
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDataError {
    Label(LabelError),
}

impl Display for ResourceDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceDataError::Label(err) => err.fmt(f),
        }
    }
}

impl Error for ResourceDataError {}

impl From<LabelError> for ResourceDataError {
    fn from(value: LabelError) -> Self {
        Self::Label(value)
    }
}

impl ResourceData {
    fn parse_host_info(value: &[u8]) -> Result<ResourceData, ResourceDataError> {
        let (cpu, offset) = parse_character_string(value)?;
        let (os, _) = parse_character_string(&value[offset..])?;
        Ok(Self::HostInfo { cpu, os })
    }

    fn parse_mail_exchange(value: &[u8]) -> Result<ResourceData, ResourceDataError> {
        let preference = u16::from_be_bytes(value[..2].try_into().unwrap());
        let exchange = Label::try_from(&value[2..])?;
        Ok(Self::MailExchange {
            preference,
            exchange,
        })
    }

    fn parse_address(value: &[u8]) -> Result<ResourceData, ResourceDataError> {
        let ip = Ipv4Addr::from(u32::from_be_bytes(value[..4].try_into().unwrap()));
        Ok(Self::Address(ip))
    }

    fn parse_mail_info(value: &[u8]) -> Result<ResourceData, ResourceDataError> {
        let (mailbox, offset) = parse_label(value)?;
        let error_mailbox = Label::try_from(&value[offset..])?;
        Ok(Self::MailInfo {
            mailbox,
            error_mailbox,
        })
    }

    fn parse_text(value: &[u8]) -> Result<ResourceData, ResourceDataError> {
        let mut buf = value;
        let mut text = vec![];

        while !buf.is_empty() {
            let (s, offset) = parse_character_string(value)?;
            buf = &buf[offset..];
            text.push(s);
        }

        Ok(Self::Text(text))
    }

    fn parse_soa(value: &[u8]) -> Result<ResourceData, ResourceDataError> {
        let mut buf;
        let (name, offset) = parse_label(value)?;
        buf = &value[offset..];
        let (mail, offset) = parse_label(buf)?;
        buf = &value[offset..];

        let serial = buf.get_u32();
        let refresh = buf.get_u32();
        let retry = buf.get_u32();
        let expire = buf.get_u32();
        let minimum = buf.get_u32();

        Ok(Self::SOA {
            name,
            mail,
            serial,
            refresh,
            retry,
            expire,
            minimum,
        })
    }
}

fn wrap_label(
    value: &[u8],
    data: fn(Label) -> ResourceData,
) -> Result<ResourceData, ResourceDataError> {
    let label = Label::try_from(value)?;
    Ok(data(label))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRecordError {
    Label(LabelError),
    Data(ResourceDataError),
    Type(UnregisteredType),
    Class(UnregisteredClass),
}

impl Display for ResourceRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ResourceRecordError::*;
        match self {
            Label(err) => err.fmt(f),
            Data(err) => err.fmt(f),
            Type(err) => err.fmt(f),
            Class(err) => err.fmt(f),
        }
    }
}

impl Error for ResourceRecordError {}

impl From<ResourceDataError> for ResourceRecordError {
    fn from(value: ResourceDataError) -> Self {
        Self::Data(value)
    }
}

impl From<LabelError> for ResourceRecordError {
    fn from(value: LabelError) -> Self {
        Self::Label(value)
    }
}

impl From<UnregisteredType> for ResourceRecordError {
    fn from(value: UnregisteredType) -> Self {
        Self::Type(value)
    }
}

impl From<UnregisteredClass> for ResourceRecordError {
    fn from(value: UnregisteredClass) -> Self {
        Self::Class(value)
    }
}

pub fn parse_resource_record(value: &[u8]) -> Result<(ResourceRecord, usize), ResourceRecordError> {
    use ResourceData::*;
    let (name, offset) = parse_label(value)?;
    let mut buf = &value[offset..];
    let mut record_offset = offset;

    let typ: ResourceType = buf.get_u16().try_into()?;
    record_offset += 2;
    let class = buf.get_u16().try_into()?;
    record_offset += 2;
    let time_to_live = buf.get_u32();
    record_offset += 4;

    let mut length = buf.get_u16() as usize;
    record_offset += 2;

    assert!(length <= buf.remaining());

    buf = &buf[..length];
    record_offset += length;

    let data = match typ {
        ResourceType::A => ResourceData::parse_address(buf)?,
        ResourceType::NS => wrap_label(buf, NameServer)?,
        ResourceType::MD => wrap_label(buf, MailDevice)?,
        ResourceType::MF => wrap_label(buf, MailForward)?,
        ResourceType::CNAME => wrap_label(buf, CanonicalName)?,
        ResourceType::SOA => ResourceData::parse_soa(buf)?,
        ResourceType::MB => wrap_label(buf, MailBox)?,
        ResourceType::MG => wrap_label(buf, MailGroup)?,
        ResourceType::MR => wrap_label(buf, MailRename)?,
        ResourceType::NULL => Null(buf.to_vec()),
        ResourceType::WKS => todo!("implement wks parser"),
        ResourceType::PTR => wrap_label(buf, Ptr)?,
        ResourceType::HINFO => ResourceData::parse_host_info(buf)?,
        ResourceType::MINFO => ResourceData::parse_mail_info(buf)?,
        ResourceType::MX => ResourceData::parse_mail_exchange(buf)?,
        ResourceType::TXT => ResourceData::parse_text(buf)?,
    };

    Ok((
        ResourceRecord {
            name,
            class,
            time_to_live,
            data,
        },
        record_offset,
    ))
}

impl TryFrom<&[u8]> for ResourceRecord {
    type Error = ResourceRecordError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        parse_resource_record(value).map(|t| t.0)
    }
}
