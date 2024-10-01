
pub fn main() {
    let mut tc = cattype::TerminalContext::new().expect("Error during terminal initialization occured");

    let quote = cattype::Quote::default();
    let config = cattype::config::Config::default();

    let mut input_stream = std::iter::repeat_with(|| tc.read_char());

    cattype::tui::run_tui(&config, &quote, &mut input_stream);
}