pub mod data;
mod prelude;
#[path = "zh-words.rs"]
pub mod zh_words;

pub use prelude::{Code, Freq, Text, Weight};
