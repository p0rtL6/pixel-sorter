use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;

use image::open;
use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use image::Rgba;
use line_drawing::Bresenham;

enum SortType {
    HorizontalLR,
    HorizontalRL,
    VerticalUD,
    VerticalDU,
    Bezier,
}

impl From<String> for SortType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "vertical-du" => {
                Self::VerticalDU
            }
            "vertical-ud" => {
                Self::VerticalUD
            }
            "horizontal-lr" => {
                Self::HorizontalLR
            }
            "horizontal-rl" => {
                Self::HorizontalRL
            }
            "bezier" => {
                Self::Bezier
            }
            _ => {
                panic!();
            }
        }
    }
}

type Intervals = Vec<Vec<(u32, u32)>>;
struct SortPixel {
    x: u32,
    y: u32,
    color: Rgba<u8>,
    lightness: f32,
}

impl SortPixel {
    fn new(image: &DynamicImage, x: u32, y: u32) -> Self {
        let color = image.get_pixel(x, y);
        SortPixel {
            x,
            y,
            color,
            lightness: get_lightness(color),
        }
    }
}

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).unwrap());
    let sort_type = SortType::from(std::env::args().nth(2).unwrap());

    if path.is_dir() {
        for file in path.read_dir().unwrap() {
            match file {
                Ok(file) => {
                    let mut image = open(file.path()).unwrap();

                    let mut output_image_name = file.file_name().to_str().unwrap().to_owned();

                    let predicate = |pixel| {
                        let lightness = get_lightness(pixel);
                        return lightness <= 0.8 && lightness >= 0.25;
                    };

                    match sort_type {
                        SortType::VerticalUD => {
                            for x in 0..image.width() {
                                let mut line: Vec<(u32, u32)> = vec![];
                                for y in 0..image.height() {
                                    line.push((x, y));
                                }
                                let intervals = get_intervals(&image, line, predicate);
                                sort(&mut image, intervals);
                            }
                            output_image_name = "VUD-".to_owned() + &output_image_name;
                        }
                        SortType::VerticalDU => {
                            image = image.rotate180();
                            for x in 0..image.width() {
                                let mut line: Vec<(u32, u32)> = vec![];
                                for y in 0..image.height() {
                                    line.push((x, y));
                                }
                                let intervals = get_intervals(&image, line, predicate);
                                sort(&mut image, intervals);
                            }
                            image = image.rotate180();
                            output_image_name = "VDU-".to_owned() + &output_image_name;
                        }
                        SortType::HorizontalLR => {
                            for y in 0..image.height() {
                                let mut line: Vec<(u32, u32)> = vec![];
                                for x in 0..image.width() {
                                    line.push((x, y));
                                }
                                let intervals = get_intervals(&image, line, predicate);
                                sort(&mut image, intervals);
                            }
                            output_image_name = "HLR-".to_owned() + &output_image_name;
                        }
                        SortType::HorizontalRL => {
                            image = image.rotate180();
                            for y in 0..image.height() {
                                let mut line: Vec<(u32, u32)> = vec![];
                                for x in 0..image.width() {
                                    line.push((x, y));
                                }
                                let intervals = get_intervals(&image, line, predicate);
                                sort(&mut image, intervals);
                            }
                            image = image.rotate180();
                            output_image_name = "HRL-".to_owned() + &output_image_name;
                        }
                        SortType::Bezier => {

                        }
                    }

                    let output_path = Path::new("output").join(output_image_name);

                    image.save(output_path).unwrap();
                }
                Err(_) => {}
            }
        }
    }
}

fn get_intervals<T>(
    image: &DynamicImage,
    line: Vec<(u32, u32)>,
    predicate: T,
) -> Vec<Vec<(u32, u32)>>
where
    T: Fn(Rgba<u8>) -> bool,
{
    let mut intervals: Vec<Vec<(u32, u32)>> = vec![];
    let mut interval: Vec<(u32, u32)> = vec![];

    for (i, (x, y)) in line.iter().enumerate() {
        let pixel = image.get_pixel(*x, *y);
        if predicate(pixel) {
            interval.push((*x, *y));
            if interval.len() > 1 && i >= line.len() - 1 {
                intervals.push(interval.clone());
                interval.clear();
            }
        } else if interval.len() > 1 {
            intervals.push(interval.clone());
            interval.clear();
        }
    }
    return intervals;
}

fn sort(image: &mut DynamicImage, intervals: Intervals) {
    for interval in intervals {
        let mut pixels: Vec<SortPixel> = interval
            .iter()
            .map(|(x, y)| SortPixel::new(&image, *x, *y))
            .collect();
        pixels.sort_unstable_by(|a, b| a.lightness.partial_cmp(&b.lightness).unwrap());
        for (index, pixel) in pixels.iter().enumerate() {
            let (x, y) = interval.get(index).unwrap();
            image.put_pixel(*x, *y, pixel.color);
        }
    }
}

fn get_lightness(pixel: Rgba<u8>) -> f32 {
    let pixel = &pixel.0[0..3];

    let max = *pixel.iter().max().unwrap() as u32;
    let min = *pixel.iter().min().unwrap() as u32;

    return (min + max) as f32 / 510.0;
}