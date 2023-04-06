#![feature(int_roundings)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(unchecked_math)]

mod colors;
mod dominant;

use crate::colors::{Ansi, Colour, Monochrome, Rgb24, Rgb565};
use clap::Parser;
use clap::ValueEnum;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel};
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::path::Path;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ColourDepth {
    Monochrome,
    Rgb24,
    Rgb16,
    Ansi,
}

#[derive(Parser)]
struct Arguments {
    /// The name of the file
    #[arg(short, long)]
    file_name: String,

    /// The width of each sample (the pixels used to determine the colour of each character).
    /// The higher this is, the smaller the overall output will be. In this case, the higher it is,
    /// the more horizontally squashed the output will be.
    #[arg(long, default_value_t = 2)]
    sample_width: u32,

    /// The height of each sample (the pixels used to determine the colour of each character).
    /// The higher this is, the smaller the overall output will be. In this case, the higher it is,
    /// the more vertically squashed it will be.
    #[arg(long, default_value_t = 3)]
    sample_height: u32,

    /// The supported colour depths. ANSI still requires truecolor support due to the nature of
    /// the way I implemented colours in the code, and I am much too lazy to change it.
    #[arg(short, long, value_enum, default_value_t = ColourDepth::Rgb24)]
    depth: ColourDepth,

    /// You can choose not to have a background colour (only one colour per character)
    #[arg(short, long, default_value_t = false)]
    no_background: bool,

    /// Apixels uses the difference of Gaussians for edge detection. This is the sigma value used in the blur
    #[arg(short, long, default_value_t = 1.0f32)]
    sigma: f32,

    /// This is the constant used for the second deviation in the difference of gaussians
    #[arg(short, long, default_value_t = 0.7f32)]
    constant: f32,
}

const EDGE_DETAIL: [u8; 92] = *b" .`'-:_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let img = image::open(Path::new(&args.file_name))?;

    let sigma = args.sigma;
    let scalar = args.constant;

    let img_blur = img.blur(scalar.powf(sigma));

    let mut img_blur = img_blur.pixels();

    let edges: ImageBuffer<image::Rgb<u8>, _> = ImageBuffer::from_vec(
        img.width(),
        img.height(),
        img.blur(sigma)
            .pixels()
            .flat_map(|pix| {
                let (_, _, image::Rgba([r1, g1, b1, _])) = img_blur.next().unwrap();
                let (_, _, image::Rgba([r2, g2, b2, _])) = pix;

                unsafe {
                    [
                        r1.unchecked_sub(r2),
                        g1.unchecked_sub(g2),
                        b1.unchecked_sub(b2),
                    ]
                }
            })
            .collect(),
    )
    .unwrap();

    let edges = DynamicImage::from(edges);

    let sample_width = args.sample_width;
    let sample_height = args.sample_height;

    let depth = args.depth;

    let img_width = img.width();
    let img_height = img.height();

    let output_text_width = img_width.div_ceil(sample_width);
    let output_text_height = img_height.div_ceil(sample_height);

    let output: String = (0..output_text_height)
        .into_par_iter()
        .flat_map(|y| {
            // Get a reference to img so that it isn't moved inside the closure
            let img = &img;

            let edges = &edges;

            // Closure must be `move` because it may outlive `y`.
            (0..output_text_width).into_par_iter().map(move |x| {
                let sample = img.crop_imm(
                    sample_width * x,
                    sample_height * y,
                    sample_width,
                    sample_height,
                );
                let edge_sample = edges.crop_imm(
                    sample_width * x,
                    sample_height * y,
                    sample_width,
                    sample_height,
                );

                let [dominant, secondary] =
                    dominant::two_most_dominant(sample.as_bytes()).map(|x| x.into_rgb());

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
                let edge_brightness: usize = edge_sample
                    .pixels()
                    .fold(0usize, |acc, (_, _, x)| acc + x.to_luma().0[0] as usize);

                let edge_avg = edge_brightness / edge_sum;

                let edge_char_idx = (edge_avg as f64 / 255.0) * (EDGE_DETAIL.len() - 1) as f64;

                let mut edge_char =
                    EDGE_DETAIL[(edge_char_idx as usize).min(EDGE_DETAIL.len() - 1)] as char;

                if args.no_background {
                    if edge_char == EDGE_DETAIL[0] as char {
                        edge_char = EDGE_DETAIL[1] as char;
                    }

                    format!(
                        "{}{}",
                        edge_char.truecolor(dominant_r, dominant_g, dominant_b),
                        if x == output_text_width - 1 {
                            '\n'
                        } else {
                            '\x00'
                        }
                    )
                } else {
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
                }
            })
        })
        .collect();

    println!("{output}");

    Ok(())
}
