#![feature(int_roundings)]

use image::io::Reader as ImageReader;
use clap::Parser;
use clap::ValueEnum;
use image::{GenericImageView, Rgb, Rgba};
use owo_colors::OwoColorize;
use rayon::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ColourDepth {
    Monochrome,
    Rgb8,
}

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    file_name: String,

    #[arg(long, default_value_t = 2)]
    sample_width: usize,

    #[arg(long, default_value_t = 3)]
    sample_height: usize,

    #[arg(short, long, value_enum, default_value_t = ColourDepth::Rgb8)]
    depth: ColourDepth
}

fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let img_reader = ImageReader::open(args.file_name)?;

    let img = img_reader.decode()?;

    let sample_width = args.sample_width;
    let sample_height = args.sample_height;

    let img_width = img.width() as usize;
    let img_height = img.height() as usize;

    let block_width = usize::from(img_width.div_ceil(sample_width));
    let block_height = usize::from(img_height.div_ceil(sample_height));

    let mut image_blocks: Vec<Vec<Rgba<u8>>> = Vec::with_capacity(block_width * block_height);

    for _ in 0..(block_height * block_width) {
        image_blocks.push(vec![])
    }

    let pixels: Vec<Rgba<u8>> = img.pixels().map(|x| x.2).collect();

    let image_blocks = image_blocks.par_iter_mut().enumerate().map(|(idx, block)| {
        let top_right = ((idx.div_floor(block_width) * img_width) + (idx * sample_width));
        println!("{top_right}");
        for y in 0..sample_height {
            for x in 0..sample_width {
                block.push(pixels[(top_right+x) + (y*img_width)]);
            }
        }

        block
    });

    // Get the average of each block
    let img_avg: Vec<Rgb<u8>> = image_blocks.map(|pixels| {
        let block_size = pixels.len();
        let (mut r_total, mut g_total, mut b_total) = (0f64, 0f64, 0f64);
        for pixel in pixels {
            let [r, g, b, a]: [f64; 4] = pixel.0.map(|x| x as f64);
            r_total += (r * (a/255.0)).powi(2);
            g_total += (g * (a/255.0)).powi(2);
            b_total += (b * (a/255.0)).powi(2);
        }
        r_total /= block_size as f64;
        g_total /= block_size as f64;
        b_total /= block_size as f64;

        r_total = r_total.sqrt();
        g_total = g_total.sqrt();
        b_total = b_total.sqrt();

        Rgb([r_total as u8, g_total as u8, b_total as u8])
    }).collect();

    let mut output = String::with_capacity(block_width * block_height * 3);

    for (idx, character) in img_avg.into_iter().enumerate() {
        let [r, g, b] = character.0;

        if idx % block_width == 0 {
            output.push('\n');
        }

        output.push_str(
            &format!("{}",
                     '#'
                         .on_truecolor(r, g, b)
                         .truecolor(r / 2, g / 2, b / 2))
        );
    }

    println!("{output}");

    Ok(())
}