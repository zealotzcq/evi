//! Fallback refine engine using keyword replacement.

use log::info;

pub struct FallbackRefineEngine {
    remove_words: Vec<String>,
}

impl FallbackRefineEngine {
    pub fn new(remove_words: Vec<String>) -> Self {
        Self { remove_words }
    }

    pub fn refine(&self, text: &str) -> String {
        let mut result = text.to_string();
        for word in &self.remove_words {
            result = result.replace(word, "");
        }
        if result != text {
            info!("FallbackRefine: '{}' -> '{}'", text, result);
        }
        result
    }
}
