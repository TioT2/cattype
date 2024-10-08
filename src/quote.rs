#[derive(Clone)]
pub struct Quote {
    pub name: String,
    pub words: Vec<String>,
}

impl Default for Quote {
    fn default() -> Self {
        Self::new(
            "1984",
            "It  was a  bright cold  day  in April,  and the  clocks were  striking thirteen.  Winston Smith,  his chin nuzzled into his breast in an effort to escape  the  vile wind, slipped quickly  through the glass doors of Victory Mansions,  though not quickly enough to prevent a swirl of gritty dust from entering along with him."
        )
    }
}

impl Quote {
    pub fn new(name: &str, source: &str) -> Self {
        Self {
            name: name.to_string(),
            words: source
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|w| w.to_string())
                .collect(),
        }
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
