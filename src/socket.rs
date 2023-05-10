extern crate libc;

use libc::{c_int, c_ushort, c_void};
use std::{io, mem, ptr};

pub struct Socket {
    fd: c_int,
}

impl Socket {
    pub fn open(domain: c_int, ty: c_int, protocol: c_int) -> io::Result<Self> {
        let fd = v2r(unsafe { libc::socket(domain, ty, protocol) })?;
        Ok(Socket { fd })
    }

    pub fn fd(&self) -> c_int {
        self.fd
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let res =
            v2r(unsafe { libc::recv(self.fd, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) })?;
        Ok(res as usize)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let res = v2r(unsafe { libc::send(self.fd, buf.as_ptr() as *mut c_void, buf.len(), 0) })?;
        Ok(res as usize)
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

pub struct PacketSocket(Socket);

impl PacketSocket {
    pub fn open() -> io::Result<Self> {
        let socket = Socket::open(libc::PF_PACKET, libc::SOCK_RAW, libc::ETH_P_ALL.to_be())?;
        Ok(PacketSocket(socket))
    }

    pub fn bind(&self, if_index: u32) -> io::Result<()> {
        let sll = libc::sockaddr_ll {
            sll_family: libc::PF_PACKET as c_ushort,
            sll_ifindex: if_index as c_int,
            sll_hatype: 0,
            sll_pkttype: 0,
            sll_halen: 0,
            sll_protocol: (libc::ETH_P_ALL as c_ushort).to_be(),
            sll_addr: [0; 8],
        };
        v2r(unsafe {
            libc::bind(
                self.0.fd(),
                ptr::addr_of!(sll) as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t,
            )
        })?;
        Ok(())
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }
}

fn v2r<T: LibcErrorIndicator>(v: T) -> io::Result<T> {
    if v.is_error() {
        Err(io::Error::last_os_error())
    } else {
        Ok(v)
    }
}

trait LibcErrorIndicator {
    fn is_error(&self) -> bool;
}

macro_rules! impl_error_indicator {
    ($($t:ident)*) => ($(impl LibcErrorIndicator for $t {
        fn is_error(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_error_indicator! { i8 i16 i32 i64 isize }
