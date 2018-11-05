#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

extern crate serde;

extern crate docopt;
extern crate env_logger;
extern crate pnet;
extern crate pnet_packet;

use std::fs::canonicalize;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddrV4;
use std::path::PathBuf;

mod packet;

use docopt::Docopt;
use packet::CovertConnection;

const USAGE: &'static str = "
Conver Send

Usage:
covert_tcp send <srcip> <dstip> <filepath>
covert_tcp (-h | --help)

Options:
  -h --help     Show help screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_send: bool,
    arg_srcip: SocketAddrV4,
    arg_dstip: SocketAddrV4,
    arg_filepath: PathBuf,
}

fn main() {
    env_logger::init();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_send {
        let mut connection = CovertConnection::new(args.arg_srcip).unwrap();

        let mut relative_path: PathBuf;
        if args.arg_filepath.is_relative() {
            relative_path = canonicalize(args.arg_filepath).unwrap();
        } else {
            relative_path = args.arg_filepath;
        }
        let mut file = File::open(relative_path).unwrap();
        let bytes = file.bytes();
        let mut seq: u32 = 1;
        for data in bytes {
            match data {
                Ok(byte) => {
                    match connection.send(args.arg_dstip, seq, byte as u16) {
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
}
