use std::fmt;

use packed_struct::prelude::*;

#[derive(PackedStruct)]
#[packed_struct(endian="lsb", size_bytes="3", bit_numbering="msb0")]
pub struct Chs {
  #[packed_field(bits="0..=7")]
  pub head: u8,
  #[packed_field(bits="8..=9")]
  pub cylinder_hi: u8,
  #[packed_field(bits="10..=15")]
  pub sector: u8,
  #[packed_field(bits="16..=23", bit_numbering="msb0")]
  pub cylinder_lo: u8,
}

impl Chs {
  pub fn cylinder(&self) -> u16 {
    self.cylinder_lo as u16 | ((self.cylinder_hi as u16) << 8)
  }
}

impl fmt::Debug for Chs {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} / {} / {}", self.cylinder(), self.head, self.sector )
  }
} 

#[derive(PackedStruct)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct Partition {
  pub active: u8,
  #[packed_field(size_bytes="3")]
  pub start_chs: Chs,
  pub part_type: u8,
  #[packed_field(size_bytes="3")]
  pub end_chs: Chs,
  pub start_lba: u32,
  pub size_lba: u32,
}

fn lba_to_mb(lba: u32) -> u32{
  (lba as u64 * 512 / 1024 / 1024) as u32
}

pub fn format_alignment(offset: u64) -> String {
  if offset == 0 {
    return "MAX".to_string();
  }
  let n = offset.trailing_zeros();
  if n >= 30 {
    format!("{} GiB", 1u64 << (n - 30))
  } else if n >= 20 {
    format!("{} MiB", 1u64 << (n - 20))
  } else if n >= 10 {
    format!("{} KiB", 1u64 << (n - 10))
  } else {
    format!("{} bytes", 1u64 << n)
  }
}

#[derive(PackedStruct, Debug)]
#[packed_struct(endian="lsb")]
pub struct Bpb {
    #[packed_field(element_size_bytes="1")]
    pub _jmp_oem: [u8; 11],
    pub bytes_per_sector: u16,    // 0x0B
    pub sectors_per_cluster: u8,   // 0x0D
    pub reserved_sectors: u16,    // 0x0E
    pub num_fats: u8,             // 0x10
    pub root_entries: u16,        // 0x11
    pub total_sectors_16: u16,    // 0x13
    pub media_type: u8,           // 0x15
    pub sectors_per_fat_16: u16,  // 0x16
    pub sectors_per_track: u16,   // 0x18
    pub num_heads: u16,           // 0x1A
    pub hidden_sectors: u32,      // 0x1C
    pub total_sectors_32: u32,    // 0x20
}

#[derive(PackedStruct, Debug)]
#[packed_struct(endian="lsb")]
pub struct Bpb32 {
    #[packed_field(element_size_bytes="1")]
    pub common: [u8; 36],
    pub sectors_per_fat_32: u32, // 0x24
    pub extended_flags: u16,      // 0x28
    pub fs_version: u16,          // 0x2A
    pub root_cluster: u32,        // 0x2C
    pub fs_info: u16,             // 0x30
    pub backup_boot_sector: u16,  // 0x32
    #[packed_field(element_size_bytes="1")]
    pub _reserved: [u8; 12],      // 0x34
    pub drive_number: u8,         // 0x40
    pub _reserved1: u8,           // 0x41
    pub boot_signature: u8,       // 0x42
    pub volume_id: u32,           // 0x43
    #[packed_field(element_size_bytes="1")]
    pub volume_label: [u8; 11],   // 0x47
    #[packed_field(element_size_bytes="1")]
    pub fs_type: [u8; 8],         // 0x52
}

pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

impl fmt::Display for FatType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FatType::Fat12 => write!(f, "FAT12"),
            FatType::Fat16 => write!(f, "FAT16"),
            FatType::Fat32 => write!(f, "FAT32"),
        }
    }
}

impl fmt::Debug for Partition {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, 
"active: {:#x}
start C/H/S: {:?}
type: {:#x}
end   C/H/S: {:?}
start lba: {} ({} MB)
size  lba: {} ({} MB)\n",
          self.active, self.start_chs, self.part_type, self.end_chs, 
          self.start_lba, lba_to_mb(self.start_lba), self.size_lba, lba_to_mb(self.size_lba) )
  }
}