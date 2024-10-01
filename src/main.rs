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

pub struct MarkedWord {
    pub chars: Vec<char>,
    pub additional_space_count: usize,
}

impl MarkedWord {
    pub fn new(data: &str) -> Self {
        Self {
            chars: data.chars().collect(),
            additional_space_count: 0,
        }
    }
}

pub struct MarkedLine {
    pub words: Vec<MarkedWord>,
}

impl MarkedLine {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }
}

pub struct MarkedQuote {
    pub alignment: usize,
    pub lines: Vec<MarkedLine>,
}

impl MarkedQuote {
    pub fn new(quote: &Quote, alignment: usize) -> Self {
        let mut lines: Vec<MarkedLine> = Vec::new();
        let mut line: MarkedLine = MarkedLine::new();
        let mut curr_len = 0;

        for word in &quote.words {
            let new_len = curr_len + line.words.len() + word.len();

            if new_len > alignment {
                lines.push(line);
                line = MarkedLine::new();

                curr_len = 0;
            }

            curr_len += word.len();
            line.words.push(MarkedWord::new(word));
        }
        lines.push(line);

        for line in &mut lines {
            let base_len = line.words.iter().map(|w| w.chars.len()).sum::<usize>() + line.words.len() - 1;
            let word_count  = line.words.len();
            let additional_space_count = alignment - base_len;

            let dadw = additional_space_count as f32 / (word_count as f32 - 1.0);
            let mut wcounter = 0.0f32;

            for word in &mut line.words {
                word.additional_space_count = ((wcounter + dadw).round() - wcounter.round()) as usize;
                wcounter += dadw;
            }
        }

        Self { alignment, lines }
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

    let marked = MarkedQuote::new(&quote, 60);

    print!("{}{}", ansi_set_cursor_position!(5, 4), untyped_ansi);
    for line in &marked.lines {
        for word in &line.words {
            for ch in &word.chars {
                print!("{ch}");
            }
            print!("{:space_count$}", "", space_count = word.additional_space_count + 1);
        }
        print!("\n    ");
    }
    print!("{}{}", ansi_set_cursor_position!(5, 4), correct_ansi);

    let mut line_iter = marked.lines.iter();
    let mut line = line_iter.next().unwrap();
    let mut word_iter = line.words.iter();
    let mut word = word_iter.next().unwrap();
    let mut char_index = 0;

    enum ExitStatus {
        Ok,
        Error,
    }

    let status = 'main_loop: loop {
        let actual_character = std::iter::repeat_with(|| tc.read_char())
            .filter(|ch| ch.is_alphanumeric() || ch.is_ascii_punctuation() || *ch == ' ' || *ch == '\x7F')
            .take(1)
            .collect::<Vec<char>>()
            .first()
            .copied()
            .unwrap();

        if actual_character == '`' {
            break 'main_loop ExitStatus::Error;
        }

        let required_character_opt = word.chars.get(char_index).copied();

        match actual_character {
            ' ' => {
                if let Some(required_character) = required_character_opt {
                    // error, fill all next by errorsign
                    print!("{}{}", untyped_ansi, required_character);
                    for index in word.chars.iter().skip(char_index + 1) {
                        print!("{}", index);
                    }
                    print!("{}", correct_ansi);
                }

                // print spaces
                for _ in 0..word.additional_space_count + 1 {
                    print!(" ");
                }

                // get next word
                if let Some(next_word) = word_iter.next() {
                    word = next_word;
                    char_index = 0;
    
                } else {
                    if let Some(next_line) = line_iter.next() {
                        line = next_line;
                        word_iter = line.words.iter();
    
                        word = word_iter.next().unwrap();
                        char_index = 0;
    
                        print!("\n    ");
                    } else {
                        break 'main_loop ExitStatus::Ok;
                    }
                }
            }
            '\x7F' => {
                if char_index > 0 {
                    char_index -= 1;
                    let ch = word.chars.get(char_index).unwrap();
                    print!("\x1B[D{}{}\x1b[D", untyped_ansi, ch);
                }

            }
            _ => {
                if let Some(required_character) = required_character_opt {
                    if actual_character == required_character {
                        print!("{}{}", correct_ansi, actual_character);
                    } else {
                        print!("{}{}", incorrect_ansi, required_character);
                    }
                    char_index += 1;
                }
            }
        }
    };

    print!("{}{}", ansi_set_cursor_position!(5, 6 + marked.lines.len()), interface_ansi);

    match status {
        ExitStatus::Ok => {
            println!("Ok");
        }
        ExitStatus::Error => {
            println!("Finished...");
        }
    }
}
