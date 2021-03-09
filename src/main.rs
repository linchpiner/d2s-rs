#[macro_use]
extern crate num_derive;

use std::fmt::Debug;
use std::path::Path;
use std::u16;
use std::u32;

use clap::{crate_authors, crate_version, Arg, App};
use failure;
use failure::{format_err, Error};
use num;

use file::FileOffset;
use stats::{Stats, StatsInfo, StatsKind};

mod file;
mod stats;

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
enum CharacterClass {
    Amazon = 0,
    Sorceress = 1,
    Necromancer = 2,
    Paladin = 3,
    Barbarian = 4,
    Druid = 5,
    Assassin = 6,
}

struct D2SaveFile {
    data: Vec<u8>,
    stats: Stats,
    skills_offset: usize,
}

impl Clone for D2SaveFile {
    fn clone(&self) -> Self {
        D2SaveFile {
            data: self.data.clone(),
            stats: self.stats.clone(),
            skills_offset: self.skills_offset,
        }
    }
}

impl D2SaveFile {

    // public methods

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read(path)?;
        if let Some(skills_offset) = D2SaveFile::skills_offset(&data) {
            let stats_offset = FileOffset::CharacterStats as usize;
            let stats = Stats::load(&data.as_slice()[stats_offset..skills_offset]);
            Ok(D2SaveFile { data, stats, skills_offset })
        } else {
            Err(format_err!("Skills section not found, invalid file format?"))
        }
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> ::std::io::Result<()> {
        self.set(FileOffset::Checksum, self.file_checksum());
        std::fs::write(path, &self.data)
    }

    pub fn header(&self) -> u32 {
        self.get_long(FileOffset::Header)
    }

    pub fn version(&self) -> u32 {
        self.get_long(FileOffset::Version)
    }

    pub fn size(&self) -> u32 {
        self.get_long(FileOffset::Size)
    }

    pub fn checksum(&self) -> u32 {
        self.get_long(FileOffset::Checksum)
    }

    pub fn level(&self) -> u8 {
        self.get_byte(FileOffset::Level)
    }

    pub fn character_class(&self) -> Option<CharacterClass> {
        num::FromPrimitive::from_u8(self.get_byte(FileOffset::Class))
    }

    pub fn file_size(&self) -> usize {
        self.data.len()
    }

    pub fn file_checksum(&self) -> u32 {
        let mut clone = self.clone();
        clone.set(FileOffset::Checksum, 0);
        let mut checksum = 0u32;
        if let Some((first, tail)) = clone.data.split_first() {
            checksum = *first as u32;
            for value in tail {
                checksum = checksum.rotate_left(1) + (*value as u32);
            }

        }
        checksum
    }

    pub fn stats(&self) -> &Vec<StatsInfo> {
        self.stats.stats()
    }

    pub fn set_stats(&mut self, kind: StatsKind, value: u32) {
        self.stats.set(kind, value);
        let stats_offset = FileOffset::CharacterStats as usize;
        self.stats.save(&mut self.data.as_mut_slice()[stats_offset..self.skills_offset]);
    }

    pub fn skills(&self) -> Vec<u8> {
        let skills_offset = self.skills_offset + 2;
        let mut skills= Vec::with_capacity(30);
        for offset in skills_offset..skills_offset + 30 {
            skills.push(self.data[offset]);
        }
        skills
    }

    // Sets a skill value (0..=99) by for the specified skill id (0..29)
    pub fn set_skill(&mut self, skill_id: usize, skill_value: u8) {
        let skill_offset = self.skills_offset + 2 + skill_id;
        self.data[skill_offset] = skill_value;
    }

    pub fn print_file_stats(&self) {
        println!("File size:     {}", self.file_size());
        println!("Size:          {}", self.size());
        println!("Header:        {:#010x}, expected: {:#010x}", self.header(), 0xaa55aa55u32);
        println!("Version:       {}", self.version());
        println!("Checksum:      {:#010x}", self.checksum());
        println!("File checksum: {:#010x}", self.file_checksum());
    }

    pub fn print_character_stats(&self) {
        println!("Level:         {}", self.level());
        if let Some(class) = self.character_class() {
            println!("Class:         {:?}", class);
        }
        for stats in self.stats() {
            println!("{:14} {}", format!("{:?}:", stats.kind()), stats.value());
        }
        println!("Skills: {:?}", self.skills());
    }

    // private methods

    fn get_byte(&self, offset: FileOffset) -> u8 {
        self.data[offset as usize]
    }

    fn get_short(&self, offset: FileOffset) -> u16 {
        self._get_short(offset as usize)
    }

    fn get_long(&self, offset: FileOffset) -> u32 {
        self._get_long(offset as usize)
    }

    fn _get_short(&self, offset: usize) -> u16 {
        (self.data[offset + 0] as u16) +
            ((self.data[offset + 1] as u16) << 8)
    }

    fn _get_long(&self, offset: usize) -> u32 {
        (self.data[offset + 0] as u32) +
            ((self.data[offset + 1] as u32) << 8) +
            ((self.data[offset + 2] as u32) << 16) +
            ((self.data[offset + 3] as u32) << 24)
    }

    fn set(&mut self, offset: FileOffset, value: u32) {
        let offset = offset as usize;
        self.data[offset + 0] = (value & 0x000000ffu32) as u8;
        self.data[offset + 1] = ((value & 0x0000ff00u32) >> 8) as u8;
        self.data[offset + 2] = ((value & 0x00ff0000u32) >> 16) as u8;
        self.data[offset + 3] = ((value & 0xff000000u32) >> 24) as u8;
    }

    // Returns offset to the start of the Skills section, which is marked by 'if' header. The file
    // has a fixed structure up to CharacterStats (offset 767), after that data is of variable
    // length.
    fn skills_offset(data: &Vec<u8>) -> Option<usize> {
        let mut offset = FileOffset::CharacterStats as usize;
        while offset <= data.len() - 2 {
            if data[offset] == 0x69 && data[offset + 1] == 0x66 {
                return Some(offset)
            }
            offset += 1;
        }
        None
    }

}

fn main() -> Result<(), Error> {
    let matches = App::new("d2s-rs")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(Arg::with_name("INPUT")
            .required(true)
            .help("Input d2s file to use")
            .index(1))
        .get_matches();

    let src_file_name = matches.value_of("INPUT").unwrap();
    let src = D2SaveFile::load(src_file_name)?;
    src.print_file_stats();
    src.print_character_stats();

    // let src_file_name = "data/Paul.d2s";
    // let dst_file_name = "data/result.d2s";
    //
    // println!("\n### File: {}", src_file_name);
    // let src = D2SaveFile::load(src_file_name)?;
    // src.print_file_stats();
    // src.print_character_stats();
    //
    // println!("\n### File: {}", dst_file_name);
    // let mut dst = src.clone();
    // dst.set_stats(StatsKind::Strength, 200);
    // dst.set_stats(StatsKind::Energy, 225);
    // dst.set_stats(StatsKind::Dexterity, 225);
    // dst.set_stats(StatsKind::Vitality, 225);
    // dst.set_stats(StatsKind::HitPoints, 31000);
    // dst.set_stats(StatsKind::MaxHealth, 31000);
    // dst.set_stats(StatsKind::Stamina, 31000);
    // dst.set_stats(StatsKind::MaxStamina, 31000);
    // dst.set_stats(StatsKind::Mana, 31000);
    // dst.set_stats(StatsKind::MaxMana, 31000);
    //
    // for skill_id in 0..30 {
    //     dst.set_skill(skill_id, 90);
    // }
    //
    // dst.save(dst_file_name)?;
    // dst.print_file_stats();
    // dst.print_character_stats();
    //
    // let src = D2SaveFile::load(dst_file_name)?;
    // println!("\n### File: {}", dst_file_name);
    // src.print_file_stats();
    // src.print_character_stats();

    Ok(())
}
