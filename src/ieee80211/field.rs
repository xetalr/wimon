use core::fmt::{self, Display, Formatter};

#[repr(transparent)]
pub struct FrameControl([u8; 2]);

impl FrameControl {
    // pub const VERSION: u16 = 0x0003;
    pub const TYPE: u16 = 0x000c;
    pub const SUB_TYPE: u16 = 0x00f0;
    // pub const TO_DS: u16 = 0x0100;
    // pub const FROM_DS: u16 = 0x0200;
    // pub const MORE_FRAGS: u16 = 0x0400;
    // pub const RETRY: u16 = 0x0800;
    // pub const POWER_MGMT: u16 = 0x1000;
    // pub const MORE_DATA: u16 = 0x2000;
    // pub const PROTECTED: u16 = 0x4000;
    pub const ORDER: u16 = 0x8000;

    pub const TYPE_MGMT: u16 = 0x0000;
    pub const SUB_TYPE_BEACON: u16 = 0x0080;
    pub const SUB_TYPE_PROBE_REQ: u16 = 0x0040;

    #[inline]
    pub fn get(&self) -> u16 {
        u16::from_le_bytes(self.0)
    }

    pub fn has_order(&self) -> bool {
        self.get() & Self::ORDER != 0
    }

    pub fn is_beacon(&self) -> bool {
        self.get() & (Self::TYPE | Self::SUB_TYPE) == (Self::TYPE_MGMT | Self::SUB_TYPE_BEACON)
    }

    pub fn is_probe_request(&self) -> bool {
        self.get() & (Self::TYPE | Self::SUB_TYPE) == (Self::TYPE_MGMT | Self::SUB_TYPE_PROBE_REQ)
    }
}

#[repr(C, packed)]
pub struct ManagementHeader {
    pub duration: DurationId,
    pub addr1: MACAddr,
    pub addr2: MACAddr,
    pub addr3: MACAddr,
    pub seq_control: SequenceControl,
}

pub type DurationId = [u8; 2];

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MACAddr([u8; 6]);

impl Display for MACAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

pub type SequenceControl = [u8; 2];
pub type HTControl = [u8; 4];

pub type Timestamp = [u8; 8];

#[repr(transparent)]
pub struct BeaconInterval([u8; 2]);

#[repr(transparent)]
pub struct Capability([u8; 2]);

impl Capability {
    pub fn has_ess(&self) -> bool {
        self.0[0] & 1 << 0 != 0
    }

    pub fn has_ibss(&self) -> bool {
        self.0[0] & 1 << 1 != 0
    }
}
