use num::{FromPrimitive, ToPrimitive};

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
pub enum StatsKind {
    Strength = 0,
    Energy = 1,
    Dexterity = 2,
    Vitality = 3,
    NewPoints = 4,
    NewSkills = 5,
    HitPoints = 6,
    MaxHealth = 7,
    Mana = 8,
    MaxMana = 9,
    Stamina = 10,
    MaxStamina = 11,
    Level = 12,
    Experience = 13,
    Gold = 14,
    GoldStash = 15,
}

#[derive(Clone)]
pub struct StatsInfo {
    kind: StatsKind,
    size: usize,
    offset: usize,
    value: u32,
}

impl StatsInfo {
    fn new(kind: StatsKind, size: usize) -> Self {
        StatsInfo{kind, size, offset: 0, value: 0}
    }

    pub fn kind(&self) -> StatsKind {
        self.kind
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

pub struct Stats {
    stats: Vec<StatsInfo>,
}

impl Clone for Stats {
    fn clone(&self) -> Self {
        Stats { stats: self.stats.clone() }
    }
}

impl Stats {
    pub fn new() -> Self {
        Stats { stats: Stats::stats_info() }
    }

    pub fn load(data: &[u8]) -> Self {
        let mut stats = Stats::new();
        Stats::parse_stats(data, &mut stats.stats);
        stats
    }

    pub fn save(&self, data: &mut [u8]) {
        let mut binary_stream = Stats::load_binary_stream(data);
        for stats in &self.stats {
            if stats.offset > 0 {
                let binary_value = format!("{:0width$b}", stats.value, width = stats.size);
                binary_stream.replace_range(stats.offset .. stats.offset + stats.size, &binary_value);
            }
        }
        Stats::save_binary_stream(data, binary_stream);
    }

    pub fn stats(&self) -> &Vec<StatsInfo> {
        &self.stats
    }

    pub fn set(&mut self, kind: StatsKind, value: u32) {
        for stats in self.stats.iter_mut() {
            if stats.kind == kind && stats.offset != 0 {
                stats.value = value;
                return;
            }
        }
        panic!("Failed to set stats {:?}", kind)
    }

    fn stats_info() -> Vec<StatsInfo> {
        let mut stats = Vec::new();
        stats.push(StatsInfo::new(StatsKind::Strength, 10));
        stats.push(StatsInfo::new(StatsKind::Energy, 10));
        stats.push(StatsInfo::new(StatsKind::Dexterity, 10));
        stats.push(StatsInfo::new(StatsKind::Vitality, 10));
        stats.push(StatsInfo::new(StatsKind::NewPoints, 10));
        stats.push(StatsInfo::new(StatsKind::NewSkills, 8));
        stats.push(StatsInfo::new(StatsKind::HitPoints, 21));
        stats.push(StatsInfo::new(StatsKind::MaxHealth, 21));
        stats.push(StatsInfo::new(StatsKind::Mana, 21));
        stats.push(StatsInfo::new(StatsKind::MaxMana, 21));
        stats.push(StatsInfo::new(StatsKind::Stamina, 21));
        stats.push(StatsInfo::new(StatsKind::MaxStamina, 21));
        stats.push(StatsInfo::new(StatsKind::Level, 7));
        stats.push(StatsInfo::new(StatsKind::Experience, 32));
        stats.push(StatsInfo::new(StatsKind::Gold, 25));
        stats.push(StatsInfo::new(StatsKind::GoldStash, 25));
        stats
    }

    fn load_binary_stream(data: &[u8]) -> String {
        let mut binary_stream = String::with_capacity(data.len() * 8);
        for offset in (0..data.len()).rev() {
            binary_stream.push_str(format!("{:08b}", data[offset]).as_str())
        }
        binary_stream
    }

    fn save_binary_stream(data: &mut [u8], binary_stream: String) {
        let data_size = data.len();
        let mut stream_offset = 0;
        let mut data_offset = 0;
        while stream_offset < binary_stream.len() {
            let binary = &binary_stream[stream_offset..stream_offset + 8];
            let byte = u8::from_str_radix(binary, 2).unwrap();
            data[data_size - data_offset - 1] = byte;
            stream_offset += 8;
            data_offset += 1;
        }
    }

    fn parse_stats(data: &[u8], stats: &mut Vec<StatsInfo>) {
        let binary_stream = Stats::load_binary_stream(data);
        let id_size = 9;
        let mut offset = binary_stream.len();
        'main: while offset >= id_size {
            offset -= id_size;
            let binary_id = &binary_stream[offset..offset + id_size];
            let id = u16::from_str_radix(binary_id, 2).unwrap();
            if let Some(kind) = FromPrimitive::from_u16(id) as Option<StatsKind> {
                //println!("Found stats id: {} ({:?})", id, kind);
                for s in stats.iter_mut() {
                    if ToPrimitive::to_u16(&s.kind) == Some(id) {
                        offset -= s.size;
                        let binary_value = &binary_stream[offset..offset + s.size];
                        let value = u32::from_str_radix(binary_value, 2).unwrap();
                        //println!("{:14} {}", format!("{:?}:", kind), value);
                        s.value = value;
                        s.offset = offset;
                        if kind == StatsKind::GoldStash {
                            return;
                        }
                        continue 'main;
                    }
                }
                break;
            }
            panic!("Found unknown stats id: {}", id);
        }
    }
}
