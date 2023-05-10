use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use image::Rgba;
use image::open;

type Intervals = Vec<Vec<(u32, u32)>>;

fn main() {
    let mut image = open("images/image.jpg").unwrap();

    let intervals = get_intervals(&image, |pixel| {
        let lightness = get_lightness(pixel);
        return lightness <= 0.8 && lightness >= 0.25;
    });

    sort(&mut image, intervals);

    image.save("output/image.jpg").unwrap();
}

fn get_lightness(pixel: Rgba<u8>) -> f32 {
    let pixel = &pixel.0[0..3];

    let max = *pixel.iter().max().unwrap() as u32;
    let min = *pixel.iter().min().unwrap() as u32;

    return (min + max) as f32 / 510.0;
}

fn sort(image: &mut DynamicImage, intervals: Intervals) {
    for y in 0..intervals.len() {
        for interval in intervals.get(y).unwrap() {

            let subimage = image.sub_image(interval.0, y.try_into().unwrap(), interval.1 - interval.0 + 1, 1);

            let mut pixels: Vec<(Rgba<u8>, f32)> = subimage.pixels().map(|(_, _, pixel)| (pixel, get_lightness(pixel))).collect();

            pixels.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for (x, (pixel, _)) in pixels.into_iter().enumerate() {
                image.put_pixel(TryInto::<u32>::try_into(x).unwrap() + interval.0, y.try_into().unwrap(), pixel);
            }
        }
    }
}

fn get_intervals<T>(image: &DynamicImage, predicate: T) -> Intervals where T: Fn(Rgba<u8>) -> bool {
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
                    intervals.get_mut(y as usize).unwrap().push((starting_index, starting_index + counter));

                }

                in_run = false;
                counter = 0;
            }
        }
        if in_run && counter != 0 {
            intervals.get_mut(y as usize).unwrap().push((starting_index, starting_index + counter));
        }
    }
    return intervals;
}