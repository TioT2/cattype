macro_rules! ansi_clear {
    () => { "\x1b[2J" };
}

macro_rules! ansi_set_cursor_position {
    ($x: expr, $y: expr) => { format!("\x1b[{};{}f", ($y), ($x)) }
}

// macro_rules! ansi_move_cursor_up    { ($dx: expr) => { format!("\x1b[{}A", $dx) } }
// macro_rules! ansi_move_cursor_down  { ($dx: expr) => { format!("\x1b[{}B", $dx) } }
macro_rules! ansi_move_cursor_right { ($dx: expr) => { format!("\x1b[{}C", $dx) } }
// macro_rules! ansi_move_cursor_left  { ($dx: expr) => { format!("\x1b[{}D", $dx) } }

pub struct MarkedWord {
    pub chars: Vec<char>,
    pub actual_chars: Vec<char>,
    pub padding: usize,
}

pub struct MarkedLine {
    pub words: Vec<MarkedWord>,
}

impl MarkedLine {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }

    pub fn minimal_len(&self) -> usize {
        0
        + self.words
            .len()
            .checked_sub(1)
            .unwrap_or(0)
        + self.words
            .iter()
            .map(|word| usize::max(word.chars.len(), word.actual_chars.len()))
            .sum::<usize>()
    }

    pub fn balance(&mut self, alignment: usize) -> Option<()> {
        let minimal_len = self.minimal_len();

        if alignment == minimal_len { return Some(()); }
        if alignment < minimal_len  { return None; }

        let additional_spaces_per_word = (alignment - minimal_len) as f32 / (self.words.len() as f32 - 1.0);
        let mut wcounter = 0.0f32;

        for word in &mut self.words {
            word.padding = ((wcounter + additional_spaces_per_word).round() - wcounter.round()) as usize;
            wcounter += additional_spaces_per_word;
        }

        Some(())
    }
}

pub struct MarkedQuote {
    pub alignment: usize,
    pub lines: Vec<MarkedLine>,
}

impl MarkedQuote {
    pub fn new(quote: &crate::Quote, alignment: usize) -> Self {
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
            line.words.push(MarkedWord {
                chars: word.chars().collect(),
            actual_chars: Vec::new(),
                padding: 0,
            });
        }
        lines.push(line);

        for line in lines.iter_mut() {
            line.balance(alignment);
        }

        Self { alignment, lines }
    }

    pub fn rebuild(&mut self, first_affected: usize) -> usize {
        let Some(potentially_affected) = self.lines.get_mut(first_affected..) else {
            return first_affected;
        };

        let mut popped = Vec::new();
        let mut last_affected = 0;

        'line_rebalance_loop: for (index, line) in potentially_affected.iter_mut().enumerate() {
            (line.words, popped) = {
                let mut words = Vec::new();
                std::mem::swap(&mut words, &mut line.words);
                (popped.into_iter().rev().chain(words.into_iter()).collect(), Vec::new())
            };

            last_affected = index;

            'pop_loop: loop {
                if line.balance(self.alignment).is_some() {
                    break 'pop_loop;
                }

                popped.push(line.words.pop().unwrap());
            }

            if popped.is_empty() {
                break 'line_rebalance_loop;
            }
        };

        if popped.is_empty() {
            return last_affected;
        }

        self.lines.push({
            let mut line = MarkedLine::new();
            line.words = popped.into_iter().rev().collect();
            _ = line.balance(self.alignment);
            line
        });

        self.lines.len() - 1
    }
}

pub fn run_tui(config: &crate::config::Config, quote: &crate::Quote, input_stream: &mut dyn Iterator<Item = char>) {
    let correct_ansi = config.colors.correct.foreground_ansi();
    let incorrect_ansi = config.colors.incorrect.foreground_ansi();
    let missed_ansi = config.colors.missed.foreground_ansi();
    let untyped_ansi = config.colors.untyped.foreground_ansi();
    let interface_ansi = config.colors.interface.foreground_ansi();

    let background_ansi = config.colors.background.background_ansi();

    // set background color
    print!("{}{}", background_ansi, ansi_clear!());

    print!("{}{}{}",
        ansi_set_cursor_position!(config.layout.name_x + 1, config.layout.name_y),
        interface_ansi,
        quote.name
    );

    let mut marked = MarkedQuote::new(
        quote,
        config.layout.alignment,
    );

    print!("{}{}", ansi_set_cursor_position!(config.layout.text_start_x + 1, config.layout.text_start_y), untyped_ansi);

    for line in &marked.lines {
        for word in &line.words {
            for ch in &word.chars {
                print!("{ch}");
            }
            print!("{:space_count$}", ' ', space_count = word.padding + 1);
        }
        print!("\n{}", ansi_move_cursor_right!(config.layout.text_start_x));
    }

    print!("{}{}", ansi_set_cursor_position!(config.layout.text_start_x + 1, config.layout.text_start_y), correct_ansi);

    let mut line_index = 0;
    let mut line = marked.lines.get(0).unwrap();
    let mut word_index = 0;
    let mut word = line.words.get(word_index).unwrap();
    let mut char_index = 0;

    enum ExitStatus {
        Ok,
        TerminatedByUser,
        InputStreamEnd,
    }

    // filter input stream
    let mut filtered_input_stream = input_stream
        .filter(|ch| false
            || ch.is_alphanumeric()
            || ch.is_ascii_punctuation()
            || *ch == ' '
            || *ch == '\x7F'
            || *ch == '\x1B'
        );

    let status = 'main_loop: loop {
        let actual_character = match filtered_input_stream.next() {
            Some(v) => v,
            None => break 'main_loop ExitStatus::InputStreamEnd,
        };

        match actual_character {
            ' ' => {
                if char_index < word.chars.len() {
                    print!("{}", missed_ansi);
                    for index in word.chars.iter().skip(char_index) {
                        print!("{}", index);
                    }
                }

                print!("{:space_count$}", "", space_count = word.padding + 1);

                let next_word = if let Some(next_word) = line.words.get(word_index + 1) {
                    word_index += 1;
                    next_word
                } else {
                    if let Some(next_line) = marked.lines.get(line_index + 1) {
                        print!("\n{}", ansi_move_cursor_right!(config.layout.text_start_x));
                        line_index += 1;
                        line = next_line;
                        word_index = 0;
                        line.words.get(0).unwrap()
                    } else {
                        break 'main_loop ExitStatus::Ok;
                    }
                };

                word = next_word;
                char_index = 0;
            }
            '\x1B' => {
                break 'main_loop ExitStatus::TerminatedByUser;
            }
            '\x7F' => {
                if char_index > 0 {
                    char_index -= 1;
                    let ch = word.chars.get(char_index).unwrap();
                    print!("\x1B[D{}{}\x1b[D", untyped_ansi, ch);
                }
            }
            _ => {
                if let Some(required_character) = word.chars.get(char_index).copied() {
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

    let cursor_position_ansi = ansi_set_cursor_position!(
        config.layout.text_start_x + 1,
        config.layout.text_start_y + marked.lines.len() + config.layout.result_offset
    );
    print!("{}{}", cursor_position_ansi, interface_ansi);

    match status {
        ExitStatus::Ok => {
            println!("Ok");
        }
        ExitStatus::TerminatedByUser => {
            println!("Terminated by user");
        }
        ExitStatus::InputStreamEnd => {
            println!("Input stream end");
        }
    }
}
