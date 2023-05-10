use crate::misc::MemCast;
use crate::socket::Socket;
use std::ffi::{CStr, CString};
use std::io::{Error, ErrorKind};
use std::{io, mem, str};

pub struct GenericNetlink(Socket);

impl GenericNetlink {
    pub fn open() -> io::Result<Self> {
        let socket = Socket::open(libc::AF_NETLINK, libc::SOCK_RAW, libc::NETLINK_GENERIC)?;
        Ok(Self(socket))
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }

    pub fn check_error(&self, reply: &[u8]) -> io::Result<()> {
        if let Some(err_code) = reply.error_code() {
            Err(Error::new(
                ErrorKind::Other,
                format!("netlink error ({})", err_code),
            ))
        } else {
            Ok(())
        }
    }
}

pub trait Controller {
    fn get_family(&self, name: &str) -> io::Result<Family>;
}

impl Controller for GenericNetlink {
    fn get_family(&self, name: &str) -> io::Result<Family> {
        let mut buf = [0u8; 4 * 1024];
        buf.create(
            libc::GENL_ID_CTRL as u16,
            libc::NLM_F_REQUEST as u16,
            1,
            0,
            libc::CTRL_CMD_GETFAMILY as u8,
            2,
        );
        buf.put_attr(
            Family::ATTR_NAME,
            CString::new(name).unwrap().as_bytes_with_nul(),
        );
        let req = &buf[..buf.header().len as usize];
        self.send(req)?;
        let reply_len = self.recv(&mut buf[..])?;
        let reply = &buf[..reply_len];
        self.check_error(reply)?;
        Ok(Family::from(reply.attrs()))
    }
}

#[repr(C)]
#[derive(Debug)]
struct MsgHdr {
    len: u32,
    r#type: u16,
    flags: u16,
    seq: u32,
    pid: u32,
}

#[repr(C)]
#[derive(Debug)]
struct GeMsgHdr {
    cmd: u8,
    version: u8,
    reserved: u16,
}

macro_rules! align {
    ($len:ident) => {
        ($len + 4 - 1) & !(4 - 1)
    };
}

trait GeMsg {
    fn header_mut(&mut self) -> &mut MsgHdr;
    fn header(&self) -> &MsgHdr;
    fn generic_header_mut(&mut self) -> &mut GeMsgHdr;
    fn generic_header(&self) -> &GeMsgHdr;
    fn create(&mut self, ty: u16, flags: u16, seq: u32, pid: u32, cmd: u8, version: u8);
    fn put_attr(&mut self, ty: u16, data: &[u8]);
    fn attrs(&self) -> AttrIter;
    fn error_code(&self) -> Option<i32>;
}

impl GeMsg for [u8] {
    fn header_mut(&mut self) -> &mut MsgHdr {
        self.cast_mut()
    }

    fn header(&self) -> &MsgHdr {
        self.cast_ref()
    }

    fn generic_header_mut(&mut self) -> &mut GeMsgHdr {
        self[mem::size_of::<MsgHdr>()..].cast_mut()
    }

    fn generic_header(&self) -> &GeMsgHdr {
        self[mem::size_of::<MsgHdr>()..].cast_ref()
    }

    fn create(&mut self, ty: u16, flags: u16, seq: u32, pid: u32, cmd: u8, version: u8) {
        let len = mem::size_of::<MsgHdr>() + mem::size_of::<GeMsgHdr>();
        let hdr = self.header_mut();
        hdr.r#type = ty;
        hdr.flags = flags;
        hdr.seq = seq;
        hdr.pid = pid;
        hdr.len = len as u32;
        let ge_hdr = self.generic_header_mut();
        ge_hdr.cmd = cmd;
        ge_hdr.version = version;
        ge_hdr.reserved = 0;
    }

    fn put_attr(&mut self, ty: u16, data: &[u8]) {
        let msg_len = self.header().len as usize;
        let attr = &mut self[msg_len..];
        let len = 4 + data.len();
        attr[..2].copy_from_slice(&(len as u16).to_ne_bytes());
        attr[2..4].copy_from_slice(&ty.to_ne_bytes());
        attr[4..len].copy_from_slice(data);
        let pad_len = align!(len);
        attr[len..pad_len].fill(0);
        self.header_mut().len += pad_len as u32;
    }

    fn attrs(&self) -> AttrIter {
        let msg_len = self.header().len as usize;
        let hdr_len = mem::size_of::<MsgHdr>() + mem::size_of::<GeMsgHdr>();
        AttrIter(&self[hdr_len..msg_len])
    }

    fn error_code(&self) -> Option<i32> {
        if self.header().r#type == libc::NLMSG_ERROR as u16 {
            Some(*self[mem::size_of::<MsgHdr>()..].cast_ref::<i32>())
        } else {
            None
        }
    }
}

pub struct AttrIter<'a>(&'a [u8]);

impl<'a> Iterator for AttrIter<'a> {
    type Item = (u16, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() < 4 {
            return None;
        }
        let len = *self.0.cast_ref::<u16>() as usize;
        let ty = *self.0[2..].cast_ref::<u16>();
        let data = &self.0[4..len];
        let pad_len = align!(len);
        self.0 = &self.0[pad_len..];
        Some((ty, data))
    }
}

#[derive(Debug)]
pub struct Family {
    id: u16,
    name: String,
    version: u32,
}

impl Family {
    const ATTR_ID: u16 = 1;
    const ATTR_NAME: u16 = 2;
    const ATTR_VERSION: u16 = 3;
}

impl From<AttrIter<'_>> for Family {
    fn from(iter: AttrIter) -> Self {
        let mut family = Family {
            id: 0,
            name: String::new(),
            version: 0,
        };
        for (ty, data) in iter {
            match ty {
                Family::ATTR_ID => family.id = *data.cast_ref::<u16>(),
                Family::ATTR_NAME => {
                    family.name = unsafe { CStr::from_bytes_with_nul_unchecked(data) }
                        .to_str()
                        .unwrap()
                        .to_owned()
                }
                Family::ATTR_VERSION => family.version = *data.cast_ref::<u32>(),
                _ => (),
            }
        }
        family
    }
}

pub struct NL80211 {
    genl: GenericNetlink,
    family: Family,
}

impl NL80211 {
    const CMD_GET_INTERFACE: u8 = 5;

    pub fn open() -> io::Result<Self> {
        let genl = GenericNetlink::open()?;
        let family = genl.get_family("nl80211")?;
        Ok(Self { genl, family })
    }

    pub fn get_interface(&self, idx: u32) -> io::Result<Interface80211> {
        let mut buf = [0u8; 4 * 1024];
        buf.create(
            self.family.id,
            libc::NLM_F_REQUEST as u16,
            1,
            0,
            Self::CMD_GET_INTERFACE,
            1,
        );
        buf.put_attr(Interface80211::ATTR_IFINDEX, &idx.to_ne_bytes());
        let req = &buf[..buf.header().len as usize];
        self.genl.send(req)?;
        let reply_len = self.genl.recv(&mut buf[..])?;
        let reply = &buf[..reply_len];
        self.genl.check_error(reply)?;
        Ok(Interface80211::from(reply.attrs()))
    }
}

#[derive(Debug)]
pub struct Interface80211 {
    pub index: u32,
    pub name: String,
    pub r#type: InterfaceType80211,
}

impl Interface80211 {
    const ATTR_IFINDEX: u16 = 3;
    const ATTR_IFNAME: u16 = 4;
    const ATTR_IFTYPE: u16 = 5;
}

impl From<AttrIter<'_>> for Interface80211 {
    fn from(iter: AttrIter) -> Self {
        let mut iface = Interface80211 {
            index: 0,
            name: String::new(),
            r#type: InterfaceType80211::Unspecified,
        };
        for (ty, data) in iter {
            match ty {
                Interface80211::ATTR_IFINDEX => iface.index = *data.cast_ref::<u32>(),
                Interface80211::ATTR_IFNAME => {
                    iface.name = CStr::from_bytes_with_nul(data)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned();
                }
                Interface80211::ATTR_IFTYPE => iface.r#type = InterfaceType80211::from(data),
                _ => (),
            }
        }
        iface
    }
}

#[derive(Debug, PartialEq)]
pub enum InterfaceType80211 {
    Unspecified,
    AdHoc,
    Station,
    AP,
    ApVlan,
    WDS,
    Monitor,
    MeshPoint,
    P2pClient,
    P2pGo,
    P2pDevice,
    OCB,
    NAN,
    Unknown,
}

impl From<&[u8]> for InterfaceType80211 {
    fn from(x: &[u8]) -> Self {
        use InterfaceType80211::*;
        match *x.cast_ref::<u32>() {
            0 => Unspecified,
            1 => AdHoc,
            2 => Station,
            3 => AP,
            4 => ApVlan,
            5 => WDS,
            6 => Monitor,
            7 => MeshPoint,
            8 => P2pClient,
            9 => P2pGo,
            10 => P2pDevice,
            11 => OCB,
            12 => NAN,
            _ => Unknown,
        }
    }
}
