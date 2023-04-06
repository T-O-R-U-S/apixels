use std::collections::HashMap;
use color_thief::{Color as CtColor, ColorFormat, get_palette};

pub fn two_most_dominant(pixels: &[u8]) -> [CtColor; 2] {
    let pixel_count = pixels.len();

    let [secondary_r, secondary_g, secondary_b]: [u8; 3] = pixels
            .iter()
            .map(|x| *x as usize)
            .array_chunks()
            .fold(
                [0, 0, 0],
                |[r_a, g_a, b_a], [r, g, b]| {
                                [r_a + r*r, g_a + g*g, b_a + b*b]
                })
        .map(|colour_sum| ((colour_sum as f64 / pixel_count as f64).sqrt() as u8));

    let average = CtColor::new(secondary_r, secondary_g, secondary_b);

    let mut map = HashMap::new();

    // let dominant = pixels.iter().array_chunks().inspect(|[r, g, b]| {
    //     let entry = map.entry([r, g, b]).or_insert(0u32);
    //
    //     *entry += 1
    // }).map(|_| map.values().max());

    for [r, g, b] in pixels.array_chunks() {
        let entry = map.entry([*r, *g, *b]).or_insert(0u32);

        *entry += 1;
    }

    let [dom_r, dom_g, dom_b] = map
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(val, _)| val)
        .expect("Sample size too small");

    // The reason the dominant colour is averaged with the... average... is to reduce jagged edges in the image.
    let dominant = CtColor::new(((dom_r as u16 + secondary_r as u16) / 2) as u8, ((dom_g as u16 + secondary_g as u16) / 2) as u8, ((dom_b as u16 + secondary_b as u16) / 2) as u8);

    [dominant, average]
}