use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::MutablePacket;
use pnet::transport::TransportChannelType::Layer3;
use pnet::transport::{transport_channel, TransportReceiver, TransportSender};
use pnet_packet::ipv4::{Ipv4Flags, MutableIpv4Packet};
use pnet_packet::tcp::ipv4_checksum;
use pnet_packet::tcp::MutableTcpPacket;
use std::io::Error;
use std::net::{IpAddr, SocketAddrV4};

type TResult<T> = Result<T, Error>;

pub struct CovertChannel {
    tx: TransportSender,
    rx: TransportReceiver,
}

impl CovertChannel {
    pub fn create() -> TResult<Self> {
        let protocol = Layer3(IpNextHeaderProtocols::Tcp);
        let (tx, rx) = transport_channel(4096, protocol)?;
        Ok(CovertChannel { tx, rx })
    }

    pub fn send_to<'a>(
        &mut self,
        addr: SocketAddrV4,
        packet: MutableIpv4Packet<'a>,
    ) -> TResult<usize> {
        info!("Sending to : {:?}", addr);
        self.tx.send_to(packet, IpAddr::V4(addr.ip().clone()))
    }
}

pub struct CovertConnection {
    source: SocketAddrV4,
    channel: CovertChannel,
}

impl CovertConnection {
    pub fn new(selfaddr: SocketAddrV4) -> TResult<Self> {
        let channel = CovertChannel::create()?;
        Ok(CovertConnection {
            source: selfaddr,
            channel,
        })
    }

    pub fn send(&mut self, addr: SocketAddrV4, seq: u32, data: u16) -> TResult<usize> {
        let tcp_packet_len = MutableTcpPacket::minimum_packet_size();
        let ipv4_packet_len = MutableIpv4Packet::minimum_packet_size();

        let mut buffer: Vec<u8> = vec![0; tcp_packet_len + ipv4_packet_len];

        let ipv4_header_len = match MutableIpv4Packet::minimum_packet_size().checked_div(4) {
            Some(l) => l as u8,
            None => panic!("Invalid header len"),
        };

        /// Building an IPV4 packet.
        let mut ipv4_packet = MutableIpv4Packet::new(&mut buffer).unwrap();
        ipv4_packet.set_header_length(ipv4_header_len as u8);
        ipv4_packet.set_total_length((tcp_packet_len + ipv4_packet_len) as u16);
        ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
        ipv4_packet.set_identification(data);
        ipv4_packet.set_source(self.source.ip().to_owned());
        ipv4_packet.set_version(4);
        ipv4_packet.set_ttl(64);
        ipv4_packet.set_destination(addr.ip().to_owned());
        ipv4_packet.set_flags(Ipv4Flags::DontFragment);

        /// Building an TCP packet
        {
            let mut tcp_packet = MutableTcpPacket::new(ipv4_packet.payload_mut()).unwrap();
            let tcp_header_len = match MutableTcpPacket::minimum_packet_size().checked_div(4) {
                Some(l) => l as u8,
                None => panic!("Invalid header len"),
            };
            tcp_packet.set_data_offset(tcp_header_len);
            tcp_packet.set_source(self.source.port());
            tcp_packet.set_destination(addr.port());
            tcp_packet.set_acknowledgement(1);
            tcp_packet.set_sequence(seq);
            let checksum = ipv4_checksum(&tcp_packet.to_immutable(), self.source.ip(), addr.ip());
            tcp_packet.set_checksum(checksum);
        };
        info!("Packet : {:?}", ipv4_packet);
        self.channel.send_to(addr, ipv4_packet)
    }
}
