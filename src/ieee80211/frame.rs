use super::element::InfoElementIter;
use super::field::{
    BeaconInterval, Capability, DurationId, FrameControl, HTControl, MACAddr, ManagementHeader,
    SequenceControl, Timestamp,
};
use crate::misc::MemCast;
use core::mem;

pub trait Frame {
    fn control(&self) -> &FrameControl;
}

impl Frame for [u8] {
    fn control(&self) -> &FrameControl {
        self.cast_ref()
    }
}

pub trait Management {
    fn header(&self) -> &ManagementHeader;
    fn duration_id(&self) -> &DurationId;
    fn addr1(&self) -> &MACAddr;
    fn addr2(&self) -> &MACAddr;
    fn addr3(&self) -> &MACAddr;
    fn seq_control(&self) -> &SequenceControl;
    fn ht_control(&self) -> Option<&HTControl>;
    fn size_of(&self) -> usize;
    fn ra(&self) -> &MACAddr;
    fn da(&self) -> &MACAddr;
    fn sa(&self) -> &MACAddr;
    fn ta(&self) -> &MACAddr;
    fn bssid(&self) -> &MACAddr;
}

impl Management for [u8] {
    fn header(&self) -> &ManagementHeader {
        self[mem::size_of::<FrameControl>()..].cast_ref()
    }

    fn duration_id(&self) -> &DurationId {
        &self.header().duration
    }

    fn addr1(&self) -> &MACAddr {
        &self.header().addr1
    }

    fn addr2(&self) -> &MACAddr {
        &self.header().addr2
    }

    fn addr3(&self) -> &MACAddr {
        &self.header().addr3
    }

    fn seq_control(&self) -> &SequenceControl {
        &self.header().seq_control
    }

    fn ht_control(&self) -> Option<&HTControl> {
        if self.control().has_order() {
            let offset = mem::size_of::<FrameControl>() + mem::size_of::<ManagementHeader>();
            Some(self[offset..].cast_ref())
        } else {
            None
        }
    }

    fn size_of(&self) -> usize {
        let size = mem::size_of::<FrameControl>() + mem::size_of::<ManagementHeader>();
        if self.control().has_order() {
            size + mem::size_of::<HTControl>()
        } else {
            size
        }
    }

    fn ra(&self) -> &MACAddr {
        &self.addr1()
    }

    fn da(&self) -> &MACAddr {
        &self.addr1()
    }

    fn sa(&self) -> &MACAddr {
        &self.addr2()
    }

    fn ta(&self) -> &MACAddr {
        &self.addr2()
    }

    fn bssid(&self) -> &MACAddr {
        &self.addr3()
    }
}

pub trait Beacon {
    fn timestamp(&self) -> &Timestamp;
    fn interval(&self) -> &BeaconInterval;
    fn capability(&self) -> &Capability;
    fn info_elements(&self) -> InfoElementIter;
}

impl Beacon for [u8] {
    fn timestamp(&self) -> &Timestamp {
        let offset = Management::size_of(self);
        self[offset..].cast_ref()
    }

    fn interval(&self) -> &BeaconInterval {
        let offset = Management::size_of(self) + mem::size_of::<Timestamp>();
        self[offset..].cast_ref()
    }

    fn capability(&self) -> &Capability {
        let offset = Management::size_of(self)
            + mem::size_of::<BeaconInterval>()
            + mem::size_of::<Timestamp>();
        self[offset..].cast_ref()
    }

    fn info_elements(&self) -> InfoElementIter {
        let offset = Management::size_of(self)
            + mem::size_of::<BeaconInterval>()
            + mem::size_of::<Timestamp>()
            + mem::size_of::<Capability>();
        InfoElementIter::new(&self[offset..])
    }
}

pub trait ProbeRequest {
    fn info_elements(&self) -> InfoElementIter;
}

impl ProbeRequest for [u8] {
    fn info_elements(&self) -> InfoElementIter {
        let offset = Management::size_of(self);
        InfoElementIter::new(&self[offset..])
    }
}
