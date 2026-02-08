use std::{env, fs::File, io::{self, BufReader, Read, Seek, SeekFrom, Write}, process::exit};

use mbrinfo::{Partition, Bpb, Bpb32, FatType, format_alignment};
use packed_struct::{PackedStruct, PackedStructSlice};

#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

fn get_sector_size(file: &File) -> Option<u32> {
    #[cfg(target_os = "linux")]
    {
        use libc::ioctl;
        let mut sector_size: libc::c_int = 0;
        // BLKSSZGET is defined as _IO(0x12, 104)
        const BLKSSZGET: libc::c_ulong = 0x1268;
        let fd = file.as_raw_fd();
        unsafe {
            if ioctl(fd, BLKSSZGET, &mut sector_size) == 0 {
                return Some(sector_size as u32);
            }
        }
    }
    None
}

fn ask_sector_size() -> u32 {
    print!("Enter sector size [512]: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    if input.is_empty() {
        512
    } else {
        input.parse().unwrap_or(512)
    }
}

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        println!("Usage: mbrinfo mbr.bin");
        exit(-1);
    }
    args.next();

    let file_path = args.next().expect("checked length");
    let file = File::open(&file_path).expect("Error opening file");

    let sector_size = get_sector_size(&file).unwrap_or_else(ask_sector_size);
    println!("Using sector size: {}\n", sector_size);

    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(446)).expect("Error seeking");

    let mut buf = [0; 16];
    let mut partitions = Vec::new();

    for _ in 0..4 {
        reader.read_exact(&mut buf).expect("Error reading");
        if buf.iter().all(|&b| b == 0) {
            partitions.push(None);
            continue;
        }
        let part = Partition::unpack(&buf).expect("unpack");
        partitions.push(Some(part));
    }

    for (i, part_opt) in partitions.into_iter().enumerate() {
        println!("Partition {i}:");
        let part = if let Some(p) = part_opt {
            p
        } else {
            println!("Zero\n");
            continue;
        };

        println!("{part:?}");

        // Alignment of the partition itself
        let part_offset = part.start_lba as u64 * sector_size as u64;
        println!("Partition Alignment: {}", format_alignment(part_offset));

        // Try to analyze FAT
        analyze_fat(&mut reader, &part, part_offset);
        println!();
    }
}

fn analyze_fat<R: Read + Seek>(reader: &mut R, part: &Partition, part_offset: u64) {
    if let Err(_) = reader.seek(SeekFrom::Start(part_offset)) {
        return;
    }

    let mut boot_sector = [0u8; 512];
    if let Err(_) = reader.read_exact(&mut boot_sector) {
        return;
    }

    // Check signature
    if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
        return;
    }

    let bpb = if let Ok(b) = Bpb::unpack_from_slice(&boot_sector[0..36]) {
        b
    } else {
        return;
    };

    if bpb.bytes_per_sector == 0 {
        return;
    }

    // Determine FAT type
    let root_dir_sectors = ((bpb.root_entries as u32 * 32) + (bpb.bytes_per_sector as u32 - 1)) / bpb.bytes_per_sector as u32;
    let fat_size = if bpb.sectors_per_fat_16 != 0 {
        bpb.sectors_per_fat_16 as u32
    } else {
        if let Ok(bpb32) = Bpb32::unpack_from_slice(&boot_sector[0..90]) {
            bpb32.sectors_per_fat_32
        } else {
            0
        }
    };

    if fat_size == 0 {
        return;
    }

    let total_sectors = if bpb.total_sectors_16 != 0 {
        bpb.total_sectors_16 as u32
    } else {
        bpb.total_sectors_32
    };

    let data_sectors = total_sectors - (bpb.reserved_sectors as u32 + (bpb.num_fats as u32 * fat_size) + root_dir_sectors);
    if bpb.sectors_per_cluster == 0 {
        return;
    }
    let cluster_count = data_sectors / bpb.sectors_per_cluster as u32;

    let fat_type = if cluster_count < 4085 {
        FatType::Fat12
    } else if cluster_count < 65525 {
        FatType::Fat16
    } else {
        FatType::Fat32
    };

    // Check mismatch with partition type
    let is_fat_type_match = match part.part_type {
        0x01 => matches!(fat_type, FatType::Fat12),
        0x04 | 0x06 | 0x0E => matches!(fat_type, FatType::Fat16),
        0x0B | 0x0C => matches!(fat_type, FatType::Fat32),
        _ => false,
    };

    if !is_fat_type_match {
        println!("Warning: MBR partition type ({:#x}) does not match detected filesystem ({})", part.part_type, fat_type);
    }

    println!("Detected Filesystem: {}", fat_type);

    let b_per_s = bpb.bytes_per_sector as u64;

    // FATs alignment
    for f in 0..bpb.num_fats {
        let fat_rel_offset = (bpb.reserved_sectors as u64 + f as u64 * fat_size as u64) * b_per_s;
        let fat_abs_offset = part_offset + fat_rel_offset;
        println!("FAT {} Offset: relative {}, absolute {}", f, fat_rel_offset, fat_abs_offset);
        println!("  Relative Alignment: {}", format_alignment(fat_rel_offset));
        println!("  Absolute Alignment: {}", format_alignment(fat_abs_offset));
    }

    // Root Directory alignment (only FAT12/16)
    match fat_type {
        FatType::Fat12 | FatType::Fat16 => {
            let root_rel_offset = (bpb.reserved_sectors as u64 + bpb.num_fats as u64 * fat_size as u64) * b_per_s;
            let root_abs_offset = part_offset + root_rel_offset;
            println!("Root Directory Offset: relative {}, absolute {}", root_rel_offset, root_abs_offset);
            println!("  Relative Alignment: {}", format_alignment(root_rel_offset));
            println!("  Absolute Alignment: {}", format_alignment(root_abs_offset));
        }
        FatType::Fat32 => {}
    }

    // Data Area alignment
    let data_rel_offset = (bpb.reserved_sectors as u64 + bpb.num_fats as u64 * fat_size as u64 + root_dir_sectors as u64) * b_per_s;
    let data_abs_offset = part_offset + data_rel_offset;
    println!("Data Area Offset: relative {}, absolute {}", data_rel_offset, data_abs_offset);
    println!("  Relative Alignment: {}", format_alignment(data_rel_offset));
    println!("  Absolute Alignment: {}", format_alignment(data_abs_offset));
}
