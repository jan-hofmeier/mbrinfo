use std::{env, fs::File, io::{BufReader, Read, Seek, SeekFrom}, process::exit};

use mbrinfo::Partition;
use packed_struct::PackedStruct;

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        println!("Usage: mbrinfo mbr.bin");
        exit(-1);
    }
    args.next();

    let mut reader = BufReader::new(File::open(args.next().expect("checked length")).expect("Error opening file"));

    reader.seek(SeekFrom::Start(446)).expect("Error seeking");

    let mut buf = [0; 16];
    for i in 0..4 {
        println!("Partition {i}:");
        reader.read_exact(&mut buf).expect("Error reading");
        let part = Partition::unpack(&buf).expect("unpack");
        println!("{part:?}");
    }
}
