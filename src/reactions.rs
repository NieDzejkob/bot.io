//! Defines the Unicode scalars of each emoji reaction used in the bot.
pub static ARROW_LEFT: &'static str = "\u{2b05}\u{fe0f}";
pub static ARROW_RIGHT: &'static str = "\u{27a1}\u{fe0f}";

pub fn digit_as_emoji(n: u8) -> String {
    format!("{}\u{fe0f}\u{20e3}", n)
}

pub fn emoji_as_digit(emoji: &str) -> Option<u8> {
    if emoji.as_bytes().get(0)?.is_ascii_digit()
        && &emoji[1..emoji.len()] == "\u{fe0f}\u{20e3}" {
        return emoji.chars().next().unwrap().to_digit(10).map(|x| x as u8);
    } else {
        None
    }
}
