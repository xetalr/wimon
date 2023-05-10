mod ieee80211;
mod misc;
mod netlink;
mod radiotap;
mod socket;

use crate::ieee80211::field::MACAddr;
use crate::misc::BytesDisplay;
use crate::netlink::{InterfaceType80211, NL80211};
use crate::radiotap::{AntennaSignal, Channel, Field, Iter as RTapIter};
use ieee80211::element::InfoElement;
use ieee80211::frame::{Beacon, Frame, Management, ProbeRequest};
use radiotap::RadioTap;
use socket::PacketSocket;
use std::collections::HashSet;
use std::env::args;
use std::ffi::CString;
use std::fmt::{Display, Formatter};

fn main() {
    if args().len() < 2 {
        exit_failure("Usage:   wimon <interface name>");
    }
    let if_name = args().nth(1).unwrap();
    let if_name_cstr = CString::new(if_name).unwrap();
    let if_idx = unsafe { libc::if_nametoindex(if_name_cstr.as_ptr()) };
    if if_idx == 0 {
        exit_failure("Interface not found");
    }
    let nl = NL80211::open().unwrap_or_else(|err| {
        exit_failure(format!("netlink: {}", err));
    });
    let iface = nl.get_interface(if_idx).unwrap_or_else(|err| {
        exit_failure(format!("nl80211 get_interface: {}", err));
    });
    if iface.r#type != InterfaceType80211::Monitor {
        exit_failure("Interface must be in monitor mode");
    }
    let pkt_sock = PacketSocket::open().unwrap_or_else(|err| {
        exit_failure(format!("socket: {}", err));
    });
    pkt_sock.bind(if_idx as u32).unwrap_or_else(|err| {
        exit_failure(format!("bind: {}", err));
    });
    let mut beacons = HashSet::new();
    let mut probes = HashSet::new();
    let mut buf = [0u8; 8 * 1024];
    loop {
        let recv_len = pkt_sock.recv(&mut buf[..]).unwrap_or_else(|err| {
            exit_failure(format!("recv: {}", err));
        });
        if recv_len <= 0 {
            continue;
        }
        let packet = &buf[..recv_len];
        let rtap_len = RadioTap::len(packet);
        let rtap_info = RTapInfo::from(RadioTap::iter(&packet[..rtap_len]));
        let frame = &packet[rtap_len..];
        if frame.control().is_beacon() {
            handle_beacon(&rtap_info, frame, &mut beacons);
        } else if frame.control().is_probe_request() {
            handle_probe_request(&rtap_info, frame, &mut probes);
        }
    }
}

fn handle_beacon(rtap_info: &RTapInfo, frame: &[u8], beacons: &mut HashSet<MACAddr>) {
    if beacons.contains(frame.bssid()) {
        return;
    }
    match (frame.capability().has_ess(), frame.capability().has_ibss()) {
        (true, false) => print!("AP STA"),
        (false, true) => print!("Ad-hoc"),
        (true, true) => print!("Mesh STA"),
        (false, false) => print!("OCB STA"),
    }
    print!(": {}, BSSID: {}", frame.ta(), frame.bssid());
    let mut info_elements = Beacon::info_elements(frame);
    while let Some(ie) = info_elements.next() {
        match ie {
            InfoElement::SSID(ssid) => print!(", SSID: {}", BytesDisplay::from(ssid)),
            InfoElement::DSSS(channel) => print!(", channel: {}", channel),
            _ => (),
        }
    }
    println!(" ({})", rtap_info);
    beacons.insert(frame.bssid().clone());
}

fn handle_probe_request(
    rtap_info: &RTapInfo,
    frame: &[u8],
    probes: &mut HashSet<(MACAddr, Vec<u8>)>,
) {
    let mut ssid = vec![];
    let mut info_elements = ProbeRequest::info_elements(frame);
    while let Some(ie) = info_elements.next() {
        match ie {
            InfoElement::SSID(ie_ssid) => ssid = ie_ssid.to_owned(),
            _ => (),
        }
    }
    let probe = (frame.ta().clone(), ssid);
    if probes.contains(&probe) {
        return;
    }
    println!(
        "STA: {} probe SSID: {} ({})",
        probe.0,
        BytesDisplay::from(probe.1.as_slice()),
        rtap_info,
    );
    probes.insert(probe);
}

fn exit_failure<T: Display>(failure: T) -> ! {
    eprintln!("{}", failure);
    std::process::exit(1);
}

#[derive(Debug)]
struct RTapInfo {
    channel: Option<Channel>,
    signal: Option<AntennaSignal>,
}

impl Default for RTapInfo {
    fn default() -> Self {
        Self {
            channel: None,
            signal: None,
        }
    }
}

impl From<RTapIter<'_>> for RTapInfo {
    fn from(rtap_iter: RTapIter) -> Self {
        let mut this = RTapInfo::default();
        for field in rtap_iter {
            match field {
                Field::Channel(_) => this.channel = Some(Channel::try_from(field).unwrap()),
                Field::AntennaSignal(_) => {
                    this.signal = Some(AntennaSignal::try_from(field).unwrap())
                }
                _ => (),
            }
        }
        this
    }
}

impl Display for RTapInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";
        if let Some(channel) = &self.channel {
            write!(f, "{}{} MHz", sep, channel.frequency_mhz())?;
            sep = ", ";
        }
        if let Some(signal) = &self.signal {
            write!(f, "{}{} dBm", sep, signal.dbm())?;
        }
        Ok(())
    }
}
