use core::str;
use std::{char, io::{Read, Write}};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

struct TerminalContext {
    initial_termios: Termios,
    stdout: std::io::Stdout,
    stdin: std::io::Stdin,
}

impl TerminalContext {
    const STDIN_FILENO: std::ffi::c_int = 0;

    pub fn new() -> Option<Self> {
        let initial_termios = Termios::from_fd(Self::STDIN_FILENO).ok()?;
        let mut termios = initial_termios.clone();

        termios.c_lflag &= !(ICANON | ECHO);

        tcsetattr(Self::STDIN_FILENO, TCSANOW, &mut termios).ok()?;

        let stdout = std::io::stdout();
        let stdin = std::io::stdin();

        Some(Self {
            initial_termios,
            stdin,
            stdout,
        })
    }

    fn read_char_opt(&mut self) -> Option<char> {
        self.stdout.lock().flush().ok()?;

        let mut read_byte = || {
            let mut buf = [0u8; 1];
            self.stdin.read_exact(&mut buf).ok()?;
            Some(buf[0])
        };

        let first_byte = read_byte()?;

        let arr: [u8; 4] = if first_byte & 0b1000_0000 == 0 {
            [first_byte, 0, 0, 0]
        } else if first_byte & 0b1110_0000 == 0b1100_0000 {
            [first_byte, read_byte()?, 0, 0]
        } else if first_byte & 0b1111_0000 == 0b1110_0000 {
            [first_byte, read_byte()?, read_byte()?, 0]
        } else if first_byte & 0b1111_1000 == 0b1111_0000 {
            [first_byte, read_byte()?, read_byte()?, read_byte()?]
        } else {
            return None;
        };

        return str::from_utf8(&arr)
            .ok()?
            .chars()
            .next();
    }

    pub fn read_char(&mut self) -> char {
        self.read_char_opt().unwrap_or('?')
    }
}

macro_rules! ansi_clear {
    () => { "\x1b[2J" };
}

macro_rules! ansi_home {
    () => { "\x1b[H" };
}

macro_rules! ansi_set_cursor_position {
    ($x: expr, $y: expr) => {
        format!("\x1b[{};{}f", ($y), ($x))
    }
}

impl Drop for TerminalContext {
    fn drop(&mut self) {
        _ = tcsetattr(Self::STDIN_FILENO, TCSANOW, &self.initial_termios);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn foreground_ansi(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    pub fn background_ansi(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0x4C, g: 0x77, b: 0xCC }
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >>  8) & 0xFF) as u8,
            b: ((value >>  0) & 0xFF) as u8,
        }
    }
}

pub struct ColorScheme {
    pub background            : Color,
    pub interface        : Color,
    pub correct          : Color,
    pub untyped          : Color,
    pub missed : Color,
    pub incorrect        : Color,
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
    pub color_scheme: ColorScheme,
}

#[derive(Clone)]
pub struct Quote {
    pub words: Vec<String>,
}

impl Default for Quote {
    fn default() -> Self {
        Self::new("It  was a  bright cold  day  in April,  and the  clocks were  striking thirteen.  Winston Smith,  his chin nuzzled into his breast in an effort to escape  the  vile wind, slipped quickly  through the glass doors of Victory Mansions,  though not quickly enough to prevent a swirl of gritty dust from entering along with him.")
    }
}

impl Quote {
    pub fn new(source: &str) -> Self {
        let words = source
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|w| w.to_string())
            .collect();

        Self { words }
    }
}

impl std::fmt::Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for elt in &self.words {
            std::fmt::Write::write_char(f, ' ')?;
            f.write_str(elt)?;
        }

        std::fmt::Result::Ok(())
    }
}

fn main() {
    let mut tc = TerminalContext::new().expect("Error during terminal initialization occured");

    let config = Config::default();

    let correct_ansi = config.color_scheme.correct.foreground_ansi();
    let incorrect_ansi = config.color_scheme.incorrect.foreground_ansi();
    let missed_ansi = config.color_scheme.missed.foreground_ansi();
    let untyped_ansi = config.color_scheme.untyped.foreground_ansi();
    let interface_ansi = config.color_scheme.interface.foreground_ansi();

    let background_ansi = config.color_scheme.background.background_ansi();

    // set background color
    print!("{}{}", background_ansi, ansi_clear!());
    print!("{}", ansi_clear!());

    print!("{}{}1984", ansi_set_cursor_position!(4, 2), interface_ansi);

    // Print quote
    let quote = Quote::default();

    // split words in lines and then type'em
    let mut lines: Vec<Vec<&str>> = Vec::new();
    let mut line: Vec<&str> = Vec::new();

    let mut curr_len = 0;
    const MAX_LEN: usize = 60;

    for word in &quote.words {
        let new_len = curr_len + line.len() + word.len();

        if new_len > MAX_LEN {
            lines.push(line);
            line = Vec::new();

            curr_len = 0;
        }

        curr_len += word.len();
        line.push(word);
    }
    lines.push(line);

    print!("{}{}", ansi_set_cursor_position!(5, 4), untyped_ansi);
    for line in lines {
        let base_len = line.iter().map(|w| w.len()).sum::<usize>() + line.len() - 1;
        let word_count  = line.len();
        let additional_space_count = MAX_LEN - base_len;

        let dadw = additional_space_count as f32 / (word_count as f32 - 1.0);
        let mut wcounter = 0.0f32;

        for word in line {
            let space_count = ((wcounter + dadw).round() - wcounter.round()) as u32;
            wcounter += dadw;
            print!("{} ", word);
            for _ in 0..space_count {
                print!(" ");
            }
        }
        print!("\n    ");
    }

    'main_loop: loop {
        let character = tc.read_char();

        // special character handling block (e.g. exit, CLS and so on)
        'filter_block: {
            match character {
                '`' => break 'main_loop,
                '^' => print!("{}", ansi_clear!()),
                _ => break 'filter_block,
            }

            continue 'main_loop;
        }

        print!("{character}");
    }
}
