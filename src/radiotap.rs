use crate::misc::MemCast;

#[repr(C, packed)]
struct Header {
    version: u8,
    pad: u8,
    len: [u8; 2],
    present: [u8; 4],
}

pub trait RadioTap {
    fn version(&self) -> u8;
    fn len(&self) -> usize;
    fn iter(&self) -> Iter;
}

impl RadioTap for [u8] {
    fn version(&self) -> u8 {
        self.cast_ref::<Header>().version
    }

    fn len(&self) -> usize {
        u16::from_le_bytes(self.cast_ref::<Header>().len) as usize
    }

    fn iter(&self) -> Iter {
        let len = self.len();
        Iter::new(&self[..len])
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    rtap: &'a [u8],
    fields: &'a [u8],
    present: u32,
    idx: usize,
}

impl<'a> Iter<'a> {
    fn new(rtap: &'a [u8]) -> Self {
        let present_ext_count = rtap[4..]
            .chunks(4)
            .take_while(|x| u32::from_le_bytes(*x.cast_ref::<[u8; 4]>()) & 1 << 31 != 0)
            .count();
        Self {
            rtap,
            fields: &rtap[8 + 4 * present_ext_count..],
            present: u32::from_le_bytes(rtap.cast_ref::<Header>().present),
            idx: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Field<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= FIELDS.len() || self.idx >= 29 {
            return None;
        }
        if self.present & 1 << self.idx != 0 {
            let field = &FIELDS[self.idx];
            let pad_len = (self.fields.as_ptr() as usize - self.rtap.as_ptr() as usize)
                & (field.alignment - 1);
            let len = pad_len + field.size;
            let (bytes, rest) = self.fields.split_at(len);
            self.idx += 1;
            self.fields = rest;
            Some((field.fn_new)(&bytes[pad_len..]))
        } else {
            self.idx += 1;
            self.next()
        }
    }
}

struct FieldDef {
    size: usize,
    alignment: usize,
    fn_new: fn(bytes: &[u8]) -> Field,
}

const FIELDS: [FieldDef; 6] = [
    FieldDef {
        size: 8,
        alignment: 8,
        fn_new: |bytes| Field::TSFT(bytes),
    },
    FieldDef {
        size: 1,
        alignment: 1,
        fn_new: |bytes| Field::Flags(bytes),
    },
    FieldDef {
        size: 1,
        alignment: 1,
        fn_new: |bytes| Field::Rate(bytes),
    },
    FieldDef {
        size: 4,
        alignment: 2,
        fn_new: |bytes| Field::Channel(bytes),
    },
    FieldDef {
        size: 2,
        alignment: 2,
        fn_new: |bytes| Field::FHSS(bytes),
    },
    FieldDef {
        size: 1,
        alignment: 1,
        fn_new: |bytes| Field::AntennaSignal(bytes),
    },
];

#[derive(Debug)]
#[non_exhaustive]
pub enum Field<'a> {
    TSFT(&'a [u8]),
    Flags(&'a [u8]),
    Rate(&'a [u8]),
    Channel(&'a [u8]),
    FHSS(&'a [u8]),
    AntennaSignal(&'a [u8]),
}

#[derive(Debug)]
pub struct InvalidField {}

#[derive(Debug)]
pub struct AntennaSignal(i8);

impl AntennaSignal {
    pub fn dbm(&self) -> i8 {
        self.0
    }
}

impl<'a> TryFrom<Field<'a>> for AntennaSignal {
    type Error = InvalidField;

    fn try_from(value: Field<'a>) -> Result<Self, Self::Error> {
        match value {
            Field::AntennaSignal(bytes) => Ok(AntennaSignal(bytes[0] as i8)),
            _ => Err(InvalidField {}),
        }
    }
}

#[derive(Debug)]
pub struct Channel {
    frequency: u16,
    _flags: u16,
}

impl Channel {
    pub fn frequency_mhz(&self) -> u16 {
        self.frequency
    }
}

impl TryFrom<Field<'_>> for Channel {
    type Error = InvalidField;

    fn try_from(value: Field<'_>) -> Result<Self, Self::Error> {
        match value {
            Field::Channel(bytes) => Ok(Channel {
                frequency: u16::from_le_bytes(*bytes.cast_ref::<[u8; 2]>()),
                _flags: u16::from_le_bytes(*bytes[2..].cast_ref::<[u8; 2]>()),
            }),
            _ => Err(InvalidField {}),
        }
    }
}
