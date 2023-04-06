use color_thief::{Color as CtColor, ColorFormat, get_palette};

pub fn two_most_dominant(pixels: &[u8]) -> [CtColor; 2] {
    let mut colours = get_palette(pixels, ColorFormat::Rgb, 1, 2).expect("Failed to retrieve two most dominant colours");

    let dominant = colours.pop().unwrap();

    let secondary = colours.pop().unwrap_or(dominant);

    [dominant, secondary]
}