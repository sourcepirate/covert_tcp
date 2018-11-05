#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

extern crate serde;

extern crate docopt;
extern crate env_logger;
extern crate pnet;
extern crate pnet_packet;
extern crate pnet_transport;

use std::fs::canonicalize;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use std::process;

mod packet;
mod sniffer;

use docopt::Docopt;
use packet::CovertConnection;
use sniffer::PacketReciver;

const USAGE: &'static str = "
Conver Send

Usage:
covert_tcp send <srcip> <dstip> <filepath>
covert_tcp recv <iface> 

Options:
  -h --help     Show help screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_send: bool,
    cmd_recv: bool,
    arg_srcip: Option<SocketAddrV4>,
    arg_dstip: Option<SocketAddrV4>,
    arg_filepath: Option<PathBuf>,
    arg_iface: Option<String>,
}

fn main() {
    env_logger::init();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_send {
        let sourceip = args.arg_srcip.unwrap();
        let destip = args.arg_dstip.unwrap();
        let filepath = args.arg_filepath.unwrap();
        let mut connection = CovertConnection::new(sourceip).unwrap();

        let mut relative_path: PathBuf;
        if filepath.is_relative() {
            relative_path = canonicalize(filepath).unwrap();
        } else {
            relative_path = filepath;
        }
        let mut file = File::open(relative_path).unwrap();
        let bytes = file.bytes();
        let mut seq: u32 = 1;
        for data in bytes {
            match data {
                Ok(byte) => {
                    match connection.send(destip, seq, byte as u16) {
                        Ok(_) => {
                            seq += 1;
                            info!("Sent data");
                        }
                        Err(_) => error!("Errored"),
                    };
                }
                Err(_e) => {
                    info!("Errored: {:?}", _e);
                }
            }
        }
    }

    if args.cmd_recv {
        let iname = args.arg_iface.unwrap();
        let mut sniff = match PacketReciver::new(iname) {
            Ok(s) => s,
            Err(_e) => {
                error!("Unknown interface");
                process::exit(0);
            }
        };
        sniff.recv();
    }
}
