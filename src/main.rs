use std::{fs::File, io::Write};

use clap::{command, Parser};

use color_eyre::{eyre::eyre, Result};
use image::{GenericImageView, Pixel};

/// A command line tool to convert fonts from png to flf
///
/// Incoming font file must be organized 16 characters wide,
/// monospaced. The first character must be space,
/// proceeding through the characters in ascii,
/// ending with ?
///
/// Incoming font image must be 16 chars wide,
/// 6 chars tall.
///
/// Pixels must be black and white,
/// with white pixels representing the font glyphs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    input: String,

    /// Output file
    #[arg(short, long)]
    output: String,

    /// Pixel character
    #[arg(short, long, default_value = "█")]
    pixel: String,

    /// Blank character
    #[arg(short, long, default_value = " ")]
    blank: String,
}

#[derive(Debug, Clone)]
struct FigletGlyph {
    rows: Vec<String>,
}
impl FigletGlyph {
    fn new(h: u32) -> Self {
        // let blank_string = " ".repeat(w as usize);
        let blank_string = String::from("");
        let rows: Vec<String> = vec![blank_string.clone(); h as usize];
        // let rows = Vec::with_capacity(h as usize);

        Self { rows }
    }

    fn output(&self) -> String {
        let mut out = String::new();
        for i in 0..self.rows.len() {
            out.push_str(self.rows[i].as_str());
            if i == self.rows.len() - 1 {
                out.push_str("@@\n");
            } else {
                out.push_str("@\n");
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
struct FigletFont {
    // char_width: u32,
    char_height: u32,
    glyphs: Vec<FigletGlyph>,
}
impl FigletFont {
    fn new(h: u32) -> Self {
        let blank_glyph = FigletGlyph::new(h);
        Self {
            // char_width: w,
            char_height: h,
            glyphs: vec![blank_glyph; 6 * 16],
        }
    }
    fn output(&self) -> String {
        let header = format!("flf2a$ {} {} 20 -1 2\n", self.char_height, self.char_height);
        let mut out = String::from(header);
        out.push_str("Font automatically generated from rust crate png-to-figlet-font\n\n");

        for glyph in &self.glyphs {
            out.push_str(glyph.output().as_str());
        }

        let space_glyph = &self.glyphs[0];
        for _ in 0..6 {
            out.push_str(space_glyph.output().as_str());
        }

        out
    }
}

// const PIXEL_CHAR: char = '█';
// const PIXEL_CHAR: char = '▚';
// const PIXEL_CHAR: char = '▉';
// const PIXEL_CHAR: char = '▇'; // 7/8

// const PIXEL_CHAR: char = '■';

// const BLANK_CHAR: char = ' ';

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let input_file_name = args.input;
    let image = image::open(input_file_name)?;

    let width = image.width();
    if width % 16 != 0 {
        return Err(eyre!("Font file is not a width divisible by 16."));
    }

    let height = image.height();
    if height % 6 != 0 {
        return Err(eyre!("Font file is not a height divisible by 6."));
    }

    let pixel_char = match args.pixel.chars().next() {
        Some(valid_char) => valid_char,
        None => return Err(eyre!("Could not use the provided pixel.")),
    };

    let blank_char = match args.blank.chars().next() {
        Some(valid_char) => valid_char,
        None => return Err(eyre!("Could not use the provided blank.")),
    };

    let char_width = width / 16;
    let char_height = height / 6;

    println!("Detected character size: {} x {}", char_width, char_height);

    let mut font = FigletFont::new(char_height);

    let mut debug_coords: Vec<(u32, u32)> = vec![];

    for glyph_y in 0u32..6 {
        for glyph_x in 0u32..16 {
            let glyph_index = glyph_y * 16 + glyph_x;
            let glyph = &mut font.glyphs[glyph_index as usize];
            for local_y in 0u32..char_height {
                let relevant_string = &mut glyph.rows[local_y as usize];
                for local_x in 0u32..char_width {
                    let x = glyph_x * char_width + local_x;
                    let y = glyph_y * char_height + local_y;
                    debug_coords.push((x, y));

                    let pixel = image.get_pixel(x, y);
                    let luminance = pixel.to_luma().0[0];
                    if luminance != 0 {
                        relevant_string.push(pixel_char);
                    } else {
                        relevant_string.push(blank_char);
                    }
                }
            }
        }
    }

    // println!("{:?}", debug_coords);

    let mut file = File::create(&args.output)?;
    file.write_all(font.output().as_bytes())?;

    Ok(())
}
