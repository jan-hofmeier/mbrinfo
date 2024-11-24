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