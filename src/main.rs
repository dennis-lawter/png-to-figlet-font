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
}

#[derive(Clone)]
struct FigletGlyph {
    rows: Vec<String>,
}
impl FigletGlyph {
    fn new(w: u32, h: u32) -> Self {
        let blank_string = " ".repeat(w as usize);
        let rows: Vec<String> = vec![blank_string.clone(); h as usize];

        Self { rows }
    }
}

struct FigletFont {
    char_width: u32,
    char_height: u32,
    glyphs: Vec<FigletGlyph>,
}
impl FigletFont {
    fn new(w: u32, h: u32) -> Self {
        let blank_glyph = FigletGlyph::new(w, h);
        Self {
            char_width: w,
            char_height: h,
            glyphs: vec![blank_glyph; 6 * 16],
        }
    }
}

const CHAR_PIXEL: char = '█';

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

    let char_width = width / 16;
    let char_height = height / 6;

    let mut font = FigletFont::new(char_width, char_height);

    for glyph_y in 0u32..6 {
        for glyph_x in 0u32..16 {
            let glyph_index = glyph_y * 16 + glyph_x;
            for local_y in 0u32..char_height {
                for local_x in 0u32..char_width {
                    let x = glyph_x * 16 + local_x;
                    let y = glyph_y * 6 + local_y;

                    let pixel = image.get_pixel(x, y);
                    let luminance = pixel.to_luma().0[0];
                    if luminance != 0 {
                        let glyph = &mut font.glyphs[glyph_index as usize];
                        let relevant_string = &mut glyph.rows[local_y as usize];
                        relevant_string.replace_range(
                            (local_x as usize)..=(local_x as usize),
                            &String::from(CHAR_PIXEL),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
