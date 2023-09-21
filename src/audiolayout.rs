pub enum ChannelType {
    Empty,
    Mono,
    Stereo
}

impl ChannelType {
    pub fn size(&self) -> u64 {
        match self {
            ChannelType::Empty => 0,
            ChannelType::Mono => 1,
            ChannelType::Stereo => 2,
        }
    }
}

pub struct Bus {
    pub name: &'static str,
    pub channel: ChannelType
}

pub struct AudioLayout {
    pub main_input: Option<Bus>,
    pub main_output: Option<Bus>,
}