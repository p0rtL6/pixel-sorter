use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;

use image::open;
use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use image::Rgba;
use indicatif::ProgressBar;
use indicatif::ProgressState;
use indicatif::ProgressStyle;
use line_drawing::Bresenham;

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
        SortPixel { x, y, color, lightness: get_lightness(color) }
    }
}

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).unwrap());
    let direction = std::env::args().nth(2).unwrap();

    if path.is_dir() {
        for file in path.read_dir().unwrap() {
            match file {
                Ok(file) => {
                    // let progress_bar = ProgressBar::new_spinner();
                    // progress_bar.set_message("Calculating Intervals...");

                    let mut image = open(file.path()).unwrap();
                    let rotated = direction == "vertical";

                    if rotated {
                        image = image.rotate90();
                    }

                    // let intervals = get_intervals(&image, |pixel| {
                    //     let lightness = get_lightness(pixel);
                    //     return lightness <= 0.8 && lightness >= 0.25;
                    // }, &progress_bar);

                    // progress_bar.finish_and_clear();
                    // let progress_bar = ProgressBar::new(intervals.iter().map(|vec| vec.len() as u64).sum());
                    // progress_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} ({eta})")
                    // .unwrap()
                    // .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                    // .progress_chars("#>-"));
                    // progress_bar.tick();

                    let w = image.width() - 1;
                    let h = image.height() - 1;
                    sort_line(&mut image, (0, 0), (w, h));

                    // sort(&mut image, intervals, &progress_bar);

                    // progress_bar.finish();

                    if rotated {
                        image = image.rotate270();
                    }

                    let output_path = Path::new("output").join(file.file_name());

                    image.save(output_path).unwrap();
                }
                Err(_) => {}
            }
        }
    }
}

fn get_line_intervals<T>(image: &DynamicImage, line: Vec<(u32, u32)>, predicate: T) -> Vec<Vec<(u32, u32)>> 
where
    T: Fn(&(u32, u32)) -> bool,
{
    let intervals: Vec<Vec<(u32, u32)>> = line.split(predicate).collect();
    return intervals;
}

fn sort_line(image: &mut DynamicImage, start: (u32, u32), end: (u32, u32)) {
    let start: (i32, i32) = (start.0.try_into().unwrap(), start.1.try_into().unwrap());
    let end: (i32, i32) = (end.0.try_into().unwrap(), end.1.try_into().unwrap());
    let line: Vec<(u32, u32)> = Bresenham::new(start, end).map(|(x, y)| (x.try_into().unwrap(), y.try_into().unwrap())).collect();
    sort_array(image, line);
}

fn sort_array(image: &mut DynamicImage, array: Vec<(u32, u32)>) {
    let mut pixels: Vec<SortPixel> = vec![];
    for (x, y) in array.iter() {
        let pixel = SortPixel::new(image, *x, *y);
        pixels.push(pixel);
    }
    pixels.sort_unstable_by(|a, b| a.lightness.partial_cmp(&b.lightness).unwrap());
    for (index, pixel) in pixels.iter().enumerate() {
        let (x, y) = array.get(index).unwrap();
        image.put_pixel(*x, *y, pixel.color);
    }
}

fn get_lightness(pixel: Rgba<u8>) -> f32 {
    let pixel = &pixel.0[0..3];

    let max = *pixel.iter().max().unwrap() as u32;
    let min = *pixel.iter().min().unwrap() as u32;

    return (min + max) as f32 / 510.0;
}

fn sort(image: &mut DynamicImage, intervals: Intervals, progress_bar: &ProgressBar) {
    for y in 0..intervals.len() {
        for interval in intervals.get(y).unwrap() {
            let subimage = image.sub_image(
                interval.0,
                y.try_into().unwrap(),
                interval.1 - interval.0 + 1,
                1,
            );

            let mut pixels: Vec<(Rgba<u8>, f32)> = subimage
                .pixels()
                .map(|(_, _, pixel)| (pixel, get_lightness(pixel)))
                .collect();

            pixels.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for (x, (pixel, _)) in pixels.into_iter().enumerate() {
                image.put_pixel(
                    TryInto::<u32>::try_into(x).unwrap() + interval.0,
                    y.try_into().unwrap(),
                    pixel,
                );
            }
            progress_bar.inc(1);
        }
    }
}

fn get_intervals<T>(image: &DynamicImage, predicate: T, progress_bar: &ProgressBar) -> Intervals
where
    T: Fn(Rgba<u8>) -> bool,
{
    let mut intervals: Intervals = vec![vec![]; image.height().try_into().unwrap()];
    for y in 0..image.height() {
        let mut counter = 0;
        let mut starting_index = 0;
        let mut in_run = false;

        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            if predicate(pixel) {
                if in_run {
                    counter += 1;
                } else {
                    starting_index = x;
                    in_run = true;
                }
            } else if in_run {
                if counter != 0 {
                    intervals
                        .get_mut(y as usize)
                        .unwrap()
                        .push((starting_index, starting_index + counter));
                }

                in_run = false;
                counter = 0;
            }
        }
        if in_run && counter != 0 {
            intervals
                .get_mut(y as usize)
                .unwrap()
                .push((starting_index, starting_index + counter));
        }
    }
    return intervals;
}
