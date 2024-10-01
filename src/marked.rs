
pub struct MarkedWord {
    pub chars: Vec<char>,
    pub padding: usize,
}

impl MarkedWord {
    pub fn len(&self) -> usize {
        self.chars.len() + self.padding
    }
}

pub struct MarkedQuote {
    pub alignment: usize,
    pub lines: Vec<Vec<MarkedWord>>,
}

impl MarkedQuote {
    pub fn new(quote: &crate::Quote, alignment: usize, tab_size: usize) -> Self {
        let mut lines: Vec<Vec<MarkedWord>> = Vec::new();
        let mut line: Vec<MarkedWord> = Vec::new();

        let mut curr_len = tab_size;

        for word in &quote.words {
            let new_len = curr_len + line.len() + word.len();

            if new_len > alignment {
                lines.push(line);
                line = Vec::new();

                curr_len = 0;
            }

            curr_len += word.len();
            line.push(MarkedWord {
                chars: word.chars().collect(),
                padding: 0,
            });
        }
        lines.push(line);

        for (index, line) in lines.iter_mut().enumerate() {
            let padding = if index == 0 { tab_size } else { 0 };

            let base_len = padding + line.iter().map(|w| w.chars.len()).sum::<usize>() + line.len() - 1;
            let word_count  = line.len();
            let additional_space_count = alignment - base_len;

            let dadw = additional_space_count as f32 / (word_count as f32 - 1.0);
            let mut wcounter = 0.0f32;

            for word in line {
                word.padding = ((wcounter + dadw).round() - wcounter.round()) as usize;
                wcounter += dadw;
            }
        }

        Self { alignment, lines }
    }
}
