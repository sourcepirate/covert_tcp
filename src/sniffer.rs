use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use pnet::util::MacAddr;
use std::char;
use std::io::Write;
use std::net::IpAddr;

pub enum ReciverError {
    InferfaceNotFound,
    UnHandledPacket,
    PacketError,
}

type PackerResult<T> = Result<T, ReciverError>;

fn handle_ipv4_packet(interface_name: &str, ethernet: &EthernetPacket) {
    info!("New Frame");
    let header = Ipv4Packet::new(ethernet.payload());
    if let Some(header) = header {
        match char::from_u32(header.get_identification() as u32) {
            Some(ch) => info!("Received : {}", ch),
            None => info!("Recived None"),
        }
    } else {
        error!("[{}]: Malformed IPv4 Packet", interface_name);
    }
}

fn handle_ethernet_frame(interface: &NetworkInterface, ethernet: &EthernetPacket) {
    let interface_name = &interface.name[..];
    handle_ipv4_packet(interface_name, ethernet);
    // match ethernet.get_ethertype() {
    //     EtherTypes::Ipv4 => handle_ipv4_packet(interface_name, ethernet),
    //     EtherTypes::Ipv6 => {
    //         info!("Ipv6");
    //         return;
    //     }
    //     EtherTypes::Arp => {
    //         info!("Arp");
    //         return;
    //     }
    //     _ => info!(
    //         "[{}]: Unknown packet: {} > {}; ethertype: {:?} length: {}",
    //         interface_name,
    //         ethernet.get_source(),
    //         ethernet.get_destination(),
    //         ethernet.get_ethertype(),
    //         ethernet.packet().len()
    //     ),
    // }
}

pub struct PacketReciver(NetworkInterface);

impl PacketReciver {
    pub fn new(iname: String) -> PackerResult<Self> {
        let interfaces = datalink::interfaces();
        let interface_names_match = |iface: &NetworkInterface| iface.name == iname;
        let interface = interfaces.into_iter().filter(interface_names_match).next();

        match interface {
            Some(i) => Ok(PacketReciver(i)),
            None => Err(ReciverError::InferfaceNotFound),
        }
    }

    pub fn recv(&mut self) -> PackerResult<()> {
        let (_, mut rx) = match datalink::channel(&self.0, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err(ReciverError::UnHandledPacket),
            Err(e) => return Err(ReciverError::PacketError),
        };

        loop {
            let mut buf: [u8; 1600] = [0u8; 1600];
            let mut fake_ethernet_frame = MutableEthernetPacket::new(&mut buf[..]).unwrap();
            match rx.next() {
                Ok(packet) => {
                    handle_ethernet_frame(&self.0, &EthernetPacket::new(packet).unwrap());
                }
                Err(e) => panic!("packetdump: unable to receive packet: {}", e),
            }
        }

        Ok(())
    }
}
