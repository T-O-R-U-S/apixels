use color_space::{CompareEuclidean};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Rgb565 {
    pub rgb: u16
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Rgb24 {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Monochrome {
    pub brightness: u8
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum Ansi {
    Black,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    Gray,
    DarkGray,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White
}

pub trait Colour {
    fn into_rgb(&self) -> (u8, u8, u8);

    fn from_rgb(colour: Rgb24) -> Self;

    fn from_rgb8(colour: (u8, u8, u8)) -> Self
    where Self: Sized {
        Self::from_rgb(Rgb24::from(colour))
    }

    fn colour_distance(&self, colour: impl Colour) -> f64 {
        let (r1, g1, b1) = self.into_rgb();
        let (r2, g2, b2) = colour.into_rgb();

        let self_color = color_space::Rgb::new(r1 as f64, g1 as f64, b1 as f64);
        let other_color = color_space::Rgb::new(r2 as f64, g2 as f64, b2 as f64);

        let self_color = color_space::Lab::from(self_color);
        let other_color = color_space::Lab::from(other_color);

        self_color.compare_euclidean(&other_color)

        // let r_diff = Decimal::try_from(r1.abs_diff(r2) as f64).expect("Failed to convert f64 to `rust_decimal` Decimal type.");
        // let g_diff = Decimal::try_from(g1.abs_diff(g2) as f64).expect("Failed to convert f64 to `rust_decimal` Decimal type.");
        // let b_diff = Decimal::try_from(b1.abs_diff(b2) as f64).expect("Failed to convert f64 to `rust_decimal` Decimal type.");
        //
        // // This basically does 3D pythagoras to figure out the distance in the RGB colorspace.
        // let distance = (r_diff.powu(2) + g_diff.powu(2) + b_diff.powu(2)).powf(0.5);
        //
        // distance.try_into().expect("Failed conversion from `rust_decimal` Decimal type to f64")
    }
}

impl Colour for Rgb24 {
    fn into_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    fn from_rgb(colour: Rgb24) -> Self {
        colour
    }
}

impl Colour for Rgb565 {
    fn into_rgb(&self) -> (u8, u8, u8) {
        let r_i = 0b1111100000000000 & self.rgb;
        let g_i = (0b0000011111100000 & self.rgb) << 5;
        let b_i = (0b0000000000011111 & self.rgb) << 11;

        // Converts RGB565 to RGB888, according to StackOverflow
        let r = (r_i as usize * 527 + 23) >> 6;
        let g = (g_i as usize * 259 + 33) >> 6;
        let b = (b_i as usize * 527 + 23) >> 6;

        (r as u8, g as u8, b as u8)
    }

    fn from_rgb(colour: Rgb24) -> Self {
        let r = (colour.r & 0b11111000) as u16;
        let g = ((colour.g & 0b11111100) as u16) >> 5;
        let b = (colour.b as u16) >> 11;

        Self {
            rgb: r | g | b
        }
    }
}

impl Colour for Monochrome {
    fn into_rgb(&self) -> (u8, u8, u8) {
        (self.brightness, self.brightness, self.brightness)
    }

    fn from_rgb(colour: Rgb24) -> Self {
        let brightness = (colour.r as u16 + colour.g as u16 + colour.b as u16).div_floor(3);

        Monochrome {
            brightness: brightness as u8
        }
    }
}

impl Colour for Ansi {
    fn into_rgb(&self) -> (u8, u8, u8) {
        // Based off of the Ubuntu terminal colour scheme (according to Wikipedia)
        match self {
            Ansi::Black => (1, 1, 1),
            Ansi::DarkRed => (222, 56, 43),
            Ansi::DarkGreen => (57, 181, 74),
            Ansi::DarkYellow => (255, 199, 6),
            Ansi::DarkBlue => (0, 111, 184),
            Ansi::DarkMagenta => (118, 38, 113),
            Ansi::DarkCyan => (44, 181, 233),
            Ansi::Gray => (204, 204, 204),
            Ansi::DarkGray => (128, 128, 128),
            Ansi::Red => (225, 0, 0),
            Ansi::Green => (0, 255, 0),
            Ansi::Yellow => (255, 255, 0),
            Ansi::Blue => (0, 0, 255),
            Ansi::Purple => (255, 0, 255),
            Ansi::Cyan => (0, 255, 255),
            Ansi::White => (255, 255, 255),
        }
    }

    fn from_rgb(colour: Rgb24) -> Self {
        let colours = [
            Ansi::Black,
            Ansi::DarkRed,
            Ansi::DarkGreen,
            Ansi::DarkYellow,
            Ansi::DarkBlue,
            Ansi::DarkMagenta,
            Ansi::DarkCyan,
            Ansi::Gray,
            Ansi::DarkGray,
            Ansi::Red,
            Ansi::Green,
            Ansi::Yellow,
            Ansi::Blue,
            Ansi::Purple,
            Ansi::Cyan,
            Ansi::White,
        ];

        let colour = colours.into_iter().reduce(|last, current| {
            let last_distance = last.colour_distance(colour);

            let current_distance = current.colour_distance(colour);

            if current_distance < last_distance {
                current
            } else {
                last
            }
        });

        colour.unwrap()
    }
}

impl Colour for (u8, u8, u8) {
    fn into_rgb(&self) -> (u8, u8, u8) {
        *self
    }

    fn from_rgb(colour: Rgb24) -> Self {
        (colour.r, colour.g, colour.b)
    }
}

impl Colour for color_thief::Color {
    fn into_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    fn from_rgb(colour: Rgb24) -> Self {
        Self {
            r: colour.r,
            g: colour.g,
            b: colour.b
        }
    }
}

impl From<(u8, u8, u8)> for Rgb24 {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Rgb24 {
            r,
            g,
            b,
        }
    }
}