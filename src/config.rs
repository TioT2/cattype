use crate::Color;

pub struct ColorScheme {
    pub background : Color,
    pub interface  : Color,
    pub correct    : Color,
    pub untyped    : Color,
    pub missed     : Color,
    pub incorrect  : Color,
}

pub struct Layout {
    pub name_x: usize,
    pub name_y: usize,
    pub text_start_x: usize,
    pub text_start_y: usize,

    pub tab_size: usize,
    pub alignment: usize,

    pub result_offset: usize,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            name_x: 4,
            name_y: 2,
            text_start_x: 8,
            text_start_y: 4,
            tab_size: 4,
            alignment: 64,
            result_offset: 1,
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background : 0x252525.into(),
            interface  : 0xFECE00.into(),
            correct    : 0xFFFFFF.into(),
            untyped    : 0xAEADA4.into(),
            missed     : 0xDCBFCF.into(),
            incorrect  : 0xBE5046.into(),
        }
    }
}

#[derive(Default)]
pub struct Config {
    pub colors: ColorScheme,
    pub layout: Layout,
}
