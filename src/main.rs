#![feature(async_closure)]
#![feature(int_roundings)]
#![feature(trivial_bounds)]

mod colors;
mod dominant;

use std::path::Path;
use clap::Parser;
use clap::ValueEnum;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use crate::colors::{Colour, Monochrome, Rgb24, Rgb565, Ansi};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ColourDepth {
    Monochrome,
    Rgb24,
    Rgb16,
    Ansi
}

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    file_name: String,

    #[arg(long, default_value_t = 2)]
    sample_width: u32,

    #[arg(long, default_value_t = 3)]
    sample_height: u32,

    #[arg(short, long, value_enum, default_value_t = ColourDepth::Rgb24)]
    depth: ColourDepth
}

const EDGE_DETAIL: [u8; 92] = *b" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let img = image::open(Path::new(&args.file_name))?;

    let img_blur = img.blur(2.0).blur(0.75);

    let mut img_blur = img_blur.pixels();

    let edges: ImageBuffer<image::Rgb<u8>, _> = ImageBuffer::from_vec(img.width(), img.height(), img.blur(2.0).pixels().flat_map(|pix| {
        let (_, _, image::Rgba([r1, g1, b1, _])) = img_blur.next().unwrap();
        let (_, _, image::Rgba([r2, g2, b2, _])) = pix;

        [r1 - r2, g1 - g2, b1 - b2]
    }).collect()).unwrap();

    let edges = DynamicImage::from(edges);

    let sample_width = args.sample_width;
    let sample_height = args.sample_height;

    let depth = args.depth;

    let img_width = img.width();
    let img_height = img.height();

    let output_text_width = img_width.div_ceil(sample_width);
    let output_text_height = img_height.div_ceil(sample_height);

    let output: String = (0..output_text_height).into_par_iter().flat_map(|y| {
        // Get a reference to img so that it isn't moved inside the closure
        let img = &img;

        let edges = &edges;

        // Closure must be `move` because it may outlive `y`.
        (0..output_text_width).into_par_iter().map(move |x| {
            let sample = img.crop_imm(sample_width*x, sample_height*y, sample_width, sample_height);
            let edge_sample = edges.crop_imm(sample_width*x, sample_height*y, sample_width, sample_height);

            let [dominant, secondary] = dominant::two_most_dominant(sample.as_bytes()).map(|x| x.into_rgb());

            let (dominant_r, dominant_g, dominant_b) = match depth {
                ColourDepth::Monochrome => {
                    let monochrome = Monochrome::from_rgb8(dominant);

                    monochrome.into_rgb()
                }
                ColourDepth::Rgb24 => {
                    let true_color = Rgb24::from_rgb8(dominant);

                    true_color.into_rgb()
                }
                ColourDepth::Rgb16 => {
                    let rgb16_color = Rgb565::from_rgb8(dominant);

                    rgb16_color.into_rgb()
                }
                ColourDepth::Ansi => {
                    let ansi_color = Ansi::from_rgb8(dominant);

                    ansi_color.into_rgb()
                }
            };

            let (secondary_r, secondary_g, secondary_b) = match depth {
                ColourDepth::Monochrome => {
                    let monochrome = Monochrome::from_rgb8(secondary);

                    monochrome.into_rgb()
                }
                ColourDepth::Rgb24 => {
                    let true_color = Rgb24::from_rgb8(secondary);

                    true_color.into_rgb()
                }
                ColourDepth::Rgb16 => {
                    let rgb16_color = Rgb565::from_rgb8(secondary);

                    rgb16_color.into_rgb()
                }
                ColourDepth::Ansi => {
                    let ansi_color = Ansi::from_rgb8(secondary);

                    ansi_color.into_rgb()
                }
            };

            let edge_sum: usize = edge_sample.pixels().count();
            let edge_brightness: usize = edge_sample.pixels().fold(0usize, |acc, (_, _, x)| acc + x.to_luma().0[0] as usize);

            let edge_avg = edge_brightness / edge_sum;

            let edge_char_idx = (edge_avg as f64 / 255.0) * (EDGE_DETAIL.len() - 1) as f64;

            let edge_char = EDGE_DETAIL[(edge_char_idx as usize).min(EDGE_DETAIL.len() - 1)] as char;

            format!(
                "{}{}",
                edge_char
                    .truecolor(secondary_r, secondary_g, secondary_b)
                    .on_truecolor(dominant_r, dominant_g, dominant_b),
                if x == output_text_width - 1 {
                    '\n'
                } else {
                    '\x00'
                }
            )
        })
    }).collect();

    println!("{output}");

    Ok(())
}