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

pub fn run_tui(config: &crate::config::Config, quote: &crate::Quote, input_stream: &mut dyn Iterator<Item = char>) {
    let correct_ansi = config.colors.correct.foreground_ansi();
    let incorrect_ansi = config.colors.incorrect.foreground_ansi();
    let missed_ansi = config.colors.missed.foreground_ansi();
    let untyped_ansi = config.colors.untyped.foreground_ansi();
    let interface_ansi = config.colors.interface.foreground_ansi();

    let background_ansi = config.colors.background.background_ansi();

    // set background color
    print!("{}{}", background_ansi, ansi_clear!());

    print!("{}{}1984", ansi_set_cursor_position!(config.layout.name_x + 1, config.layout.name_y), interface_ansi);

    let marked = crate::marked::MarkedQuote::new(
        quote,
        config.layout.alignment,
        config.layout.tab_size
    );

    print!("{}{}", ansi_set_cursor_position!(config.layout.text_start_x + 1, config.layout.text_start_y), untyped_ansi);
    if config.layout.tab_size != 0 {
        print!("{:space_count$}", "", space_count = config.layout.tab_size);
    }
    for line in &marked.lines {
        for word in line {
            for ch in &word.chars {
                print!("{ch}");
            }
            print!("{:space_count$}", ' ', space_count = word.padding + 1);
        }
        print!("\n{}", ansi_move_cursor_right!(config.layout.text_start_x));
    }

    print!("{}{}", ansi_set_cursor_position!(config.layout.text_start_x + 1, config.layout.text_start_y), correct_ansi);
    if config.layout.tab_size != 0 {
        print!("{:space_count$}", "", space_count = config.layout.tab_size);
    }

    let mut line_iter = marked.lines.iter();
    let mut line = line_iter.next().unwrap();
    let mut word_iter = line.iter();
    let mut word = word_iter.next().unwrap();
    let mut char_index = 0;

    enum ExitStatus {
        Ok,
        Error,
        InputStreamEnd,
    }

    // filter input stream
    let mut filtered_input_stream = input_stream
        .filter(|ch| false
            || ch.is_alphanumeric()
            || ch.is_ascii_punctuation()
            || *ch == ' '
            || *ch == '\x7F'
        );

    let status = 'main_loop: loop {
        let actual_character = match filtered_input_stream.next() {
            Some(v) => v,
            None => break 'main_loop ExitStatus::InputStreamEnd,
        };

        // quit
        if actual_character == '`' {
            break 'main_loop ExitStatus::Error;
        }

        let required_character_opt = word.chars.get(char_index).copied();

        match actual_character {
            ' ' => {
                if let Some(required_character) = required_character_opt {
                    print!("{}{}", missed_ansi, required_character);
                    for index in word.chars.iter().skip(char_index + 1) {
                        print!("{}", index);
                    }
                }

                print!("{:space_count$}", "", space_count = word.padding + 1);

                // get next word
                if let Some(next_word) = word_iter.next() {
                    word = next_word;
                    char_index = 0;

                } else {
                    if let Some(next_line) = line_iter.next() {
                        line = next_line;
                        word_iter = line.iter();

                        word = word_iter.next().unwrap();
                        char_index = 0;

                        print!("\n{}", ansi_move_cursor_right!(config.layout.text_start_x));
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

    let cursor_position_ansi = ansi_set_cursor_position!(
        config.layout.text_start_x + 1,
        config.layout.text_start_y + marked.lines.len() + config.layout.result_offset
    );
    print!("{}{}", cursor_position_ansi, interface_ansi);

    match status {
        ExitStatus::Ok => {
            println!("Ok");
        }
        ExitStatus::Error => {
            println!("Finished manually");
        }
        ExitStatus::InputStreamEnd => {
            println!("Input stream end");
        }
    }
}
