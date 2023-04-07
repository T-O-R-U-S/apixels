#![feature(int_roundings)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(unchecked_math)]

mod colors;
mod dominant;

use std::path::Path;
use apixels::into_ascii_controlled;
use apixels::Arguments;
use clap::Parser;

const EDGE_DETAIL: [u8; 90] = *b" `-:_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let img = std::fs::read(Path::new(&args.file_name))?;

    let sigma = args.sigma;
    let scalar = args.constant;

    let sample_width = args.sample_width;
    let sample_height = args.sample_height;

    let output = into_ascii_controlled(img, (sigma, scalar), (sample_width, sample_height), args.no_background, args.edges, args.depth)?;

    // let edges: ImageBuffer<image::Rgb<u8>, _> = ImageBuffer::from_vec(
    //     img.width(),
    //     img.height(),
    //     img.blur(sigma)
    //         .pixels()
    //         .flat_map(|pix| unsafe {
    //             let (_, _, image::Rgba([r1, g1, b1, _])) = pix;
    //             let (_, _, image::Rgba([r2, g2, b2, _])) = img_blur.next().unwrap();
    //
    //             let r = r1.checked_sub(r2).unwrap_or_else(|| r2.unchecked_sub(r1)).saturating_mul(3);
    //             let g = g1.checked_sub(g2).unwrap_or_else(|| g2.unchecked_sub(g1)).saturating_mul(4);
    //             let b = b1.checked_sub(b2).unwrap_or_else(|| b2.unchecked_sub(b1)).saturating_mul(3);
    //
    //             [
    //                 r,
    //                 g,
    //                 b,
    //             ]
    //         })
    //         .collect(),
    // )
    // .unwrap();
    //
    // let edges = DynamicImage::from(edges);
    //
    // let sample_width = args.sample_width;
    // let sample_height = args.sample_height;
    //
    // if sample_width == 0 || sample_height == 0 {
    //     bail!("Sample width and height must be non-zero!")
    // }
    //
    // let depth = args.depth;
    //
    // let img_width = img.width();
    // let img_height = img.height();
    //
    // let output_text_width = img_width.div_ceil(sample_width);
    // let output_text_height = img_height.div_ceil(sample_height);
    //
    // let output: String = (0..output_text_height)
    //     .into_par_iter()
    //     .flat_map(|y| {
    //         // Get a reference to img so that it isn't moved inside the closure
    //         let img = &img;
    //
    //         let edges = &edges;
    //
    //         // Closure must be `move` because it may outlive `y`.
    //         (0..output_text_width).into_par_iter().map(move |x| {
    //             let edge_sample = edges.crop_imm(
    //                 sample_width * x,
    //                 sample_height * y,
    //                 sample_width,
    //                 sample_height,
    //             );
    //
    //             let sample = if !args.edges { img.crop_imm(
    //                 sample_width * x,
    //                 sample_height * y,
    //                 sample_width,
    //                 sample_height,
    //             ) } else { edge_sample.clone() };
    //
    //             let [dominant, secondary] =
    //                 dominant::two_most_dominant(sample.as_bytes()).map(|x| x.into_rgb());
    //
    //             // I could and should extract these two blocks out into a function. However, I do not care.
    //             // If you care, you can submit a PR.
    //             let (dominant_r, dominant_g, dominant_b) = match depth {
    //                 ColourDepth::Grayscale => {
    //                     let monochrome = Monochrome::from_rgb8(dominant);
    //
    //                     monochrome.into_rgb()
    //                 }
    //                 ColourDepth::Rgb24 => {
    //                     let true_color = Rgb24::from_rgb8(dominant);
    //
    //                     true_color.into_rgb()
    //                 }
    //                 ColourDepth::Rgb16 => {
    //                     let rgb16_color = Rgb565::from_rgb8(dominant);
    //
    //                     rgb16_color.into_rgb()
    //                 }
    //                 ColourDepth::Ansi => {
    //                     let ansi_color = Ansi::from_rgb8(dominant);
    //
    //                     ansi_color.into_rgb()
    //                 }
    //                 ColourDepth::None => (0, 0, 0)
    //             };
    //
    //             let (secondary_r, secondary_g, secondary_b) = match depth {
    //                 ColourDepth::Grayscale => {
    //                     let monochrome = Monochrome::from_rgb8(secondary);
    //
    //                     monochrome.into_rgb()
    //                 }
    //                 ColourDepth::Rgb24 => {
    //                     let true_color = Rgb24::from_rgb8(secondary);
    //
    //                     true_color.into_rgb()
    //                 }
    //                 ColourDepth::Rgb16 => {
    //                     let rgb16_color = Rgb565::from_rgb8(secondary);
    //
    //                     rgb16_color.into_rgb()
    //                 }
    //                 ColourDepth::Ansi => {
    //                     let ansi_color = Ansi::from_rgb8(secondary);
    //
    //                     ansi_color.into_rgb()
    //                 }
    //                 ColourDepth::None => (255, 255, 255)
    //             };
    //
    //             let edge_sum: usize = edge_sample.pixels().count();
    //             let edge_brightness: usize = edge_sample
    //                 .pixels()
    //                 .fold(0usize, |acc, (_, _, x)| acc + x.to_luma().0[0] as usize);
    //
    //             let edge_avg = edge_brightness / edge_sum;
    //
    //             let edge_char_idx = (edge_avg as f64 / 255.0) * (EDGE_DETAIL.len() - 1) as f64;
    //
    //             let mut edge_char =
    //                 EDGE_DETAIL[(edge_char_idx as usize).min(EDGE_DETAIL.len() - 1)] as char;
    //
    //             if args.no_background {
    //                 if edge_char == EDGE_DETAIL[0] as char {
    //                     edge_char = EDGE_DETAIL[1] as char;
    //                 }
    //
    //                 format!(
    //                     "{}{}",
    //                     edge_char.truecolor(dominant_r, dominant_g, dominant_b),
    //                     if x == output_text_width - 1 {
    //                         '\n'
    //                     } else {
    //                         '\x00'
    //                     }
    //                 )
    //             } else {
    //                 format!(
    //                     "{}{}",
    //                     edge_char
    //                         .truecolor(secondary_r, secondary_g, secondary_b)
    //                         .on_truecolor(dominant_r, dominant_g, dominant_b),
    //                     if x == output_text_width - 1 {
    //                         '\n'
    //                     } else {
    //                         '\x00'
    //                     }
    //                 )
    //             }
    //         })
    //     })
    //     .collect();

    println!("{output}");

    Ok(())
}
