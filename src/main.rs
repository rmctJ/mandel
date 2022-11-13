use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num::Complex;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::env;
use std::fs::File;
use std::io::Error;
use std::str::FromStr;

fn escape_time(c: Complex<f64>, retry: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..retry {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn parse_pair<F: FromStr>(s: &str, sep: char) -> Option<(F, F)> {
    match s.find(sep) {
        Some(i_sep) => match (F::from_str(&s[0..i_sep]), F::from_str(&s[i_sep + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
        None => None,
    }
}

fn parse_complext(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        _ => None,
    }
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let width = lower_right.re - upper_left.re;
    let height = upper_left.im - lower_right.im;

    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), Error> {
    let output = File::create(filename)?;

    let encoder = PngEncoder::new(output);

    encoder
        .write_image(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8)
        .unwrap();
    Ok(())
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 5 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT", args[0]);
        eprintln!(
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("Cannot parse PIXELS");
    let upper_left = parse_complext(&args[3]).expect("Cannot parse UPPERLEFT");
    let lower_right = parse_complext(&args[4]).expect("Cannot parse LOWERRIGHT");

    let pixels = (0..bounds.0 * bounds.1)
        .into_par_iter()
        .map(|id| {
            let row = id / bounds.0;
            let col = id % bounds.0;
            let point = pixel_to_point(bounds, (col, row), upper_left, lower_right);

            match escape_time(point, 255) {
                None => 0,
                Some(v) => 255 - v as u8,
            }
        })
        .collect::<Vec<_>>();

    write_image(&args[1], &pixels, bounds).unwrap();
}
