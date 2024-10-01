use core::str;
use std::io::{Read, Write};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub struct TerminalContext {
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

impl Drop for TerminalContext {
    fn drop(&mut self) {
        _ = tcsetattr(Self::STDIN_FILENO, TCSANOW, &self.initial_termios);
    }
}
