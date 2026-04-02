use crate::models::game::{CardValue, DeckType};

fn num(n: f64) -> CardValue {
    CardValue::Number(n)
}

fn text(s: &str) -> CardValue {
    CardValue::Text(s.to_string())
}

pub fn get_deck(deck_type: &DeckType) -> Vec<CardValue> {
    match deck_type {
        DeckType::Fibonacci => vec![
            num(0.0),
            num(1.0),
            num(2.0),
            num(3.0),
            num(5.0),
            num(8.0),
            num(13.0),
            num(21.0),
            num(34.0),
            num(55.0),
            num(89.0),
            text("?"),
            text("☕"),
        ],
        DeckType::Tshirt => vec![
            text("XS"),
            text("S"),
            text("M"),
            text("L"),
            text("XL"),
            text("XXL"),
            text("?"),
            text("☕"),
        ],
        DeckType::Powers2 => vec![
            num(0.0),
            num(1.0),
            num(2.0),
            num(4.0),
            num(8.0),
            num(16.0),
            num(32.0),
            num(64.0),
            text("?"),
            text("☕"),
        ],
        DeckType::Custom => vec![],
    }
}
