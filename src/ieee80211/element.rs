#[non_exhaustive]
#[repr(u8)]
pub enum InfoElementId {
    SSID = 0,
    // SupportedRates = 1,
    // FH = 2,
    DSSS = 3,
    // ...
    // RSN = 48,
    // ...
    // VendorSpecific = 221,
}

#[non_exhaustive]
pub enum InfoElement<'a> {
    Generic(GenericInfoElement<'a>),
    SSID(&'a [u8]),
    DSSS(u8),
}

pub struct GenericInfoElement<'a> {
    _id: u8,
    _data: &'a [u8],
}

pub struct InfoElementIter<'a> {
    buf: &'a [u8],
}

impl<'a> InfoElementIter<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Iterator for InfoElementIter<'a> {
    type Item = InfoElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() < 2 {
            return None;
        }
        let id = self.buf[0];
        let data_len = self.buf[1] as usize;
        let element_len = 2 + data_len;
        if self.buf.len() < element_len {
            return None;
        }
        let data = &self.buf[2..element_len];
        self.buf = &self.buf[element_len..];
        use InfoElement as IE;
        use InfoElementId as Id;
        match id {
            x if x == Id::SSID as u8 => Some(IE::SSID(data)),
            x if x == Id::DSSS as u8 => Some(IE::DSSS(data[0])),
            _ => Some(InfoElement::Generic(GenericInfoElement { _id: id, _data: data })),
        }
    }
}
