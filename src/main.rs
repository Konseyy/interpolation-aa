extern crate image;

use colors_transform::{Color, Rgb};
use image::{DynamicImage, GenericImageView, Rgba};
use std::fs;
use std::path::Path;
fn process_image(input_path: &str) -> Option<(u32, u32, DynamicImage)> {
    let img = image::open(&Path::new(input_path));
    if img.is_err() {
        println!("Error: {}", img.err().unwrap());
        return None;
    }

    let width = img.as_ref().unwrap().width();
    let height = img.as_ref().unwrap().height();

    return Some((width, height, img.as_ref().unwrap().clone()));
}

fn get_neighbor_coordinates(x: u32, y: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut neighbors: Vec<(u32, u32)> = Vec::new();
    if x > 0 {
        neighbors.push((x - 1, y));
    }
    if x < width - 1 {
        neighbors.push((x + 1, y));
    }
    if y > 0 {
        neighbors.push((x, y - 1));
    }
    if y < height - 1 {
        neighbors.push((x, y + 1));
    }
    return neighbors;
}

const INTERPOLATION_THRESHOLD: f32 = 20.0;
fn main() {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    print!("Please enter image path: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    let mut passes_input = String::new();
    let interpolation_passes: i16;
    print!("Please enter interpolation pass count (from 1 to 32): ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut passes_input)
        .expect("Did not enter a correct string");
    if let Some('\n') = passes_input.chars().next_back() {
        passes_input.pop();
    }
    if let Some('\r') = passes_input.chars().next_back() {
        passes_input.pop();
    }

    let trimmed = passes_input.trim();
    match trimmed.parse::<i16>() {
        Ok(i) => {
            if i > 32 || i < 1 {
                println!("Contrast value must be from 1 to 32");
                return;
            }
            interpolation_passes = i;
        }
        Err(..) => {
            println!("this was not an integer: {}", trimmed);
            return;
        }
    };
    let img_info_result = process_image(&s);
    if img_info_result.is_none() {
        println!("Could not process image {}", s);
        return;
    }
    let mut img_info = img_info_result.unwrap();
    let startingImg = img_info.2.clone();
    let mut new_img_points = Vec::new();
    for pass in 0..interpolation_passes {
        println!("Pass {}", pass + 1);
        new_img_points = Vec::new();
        for point in img_info.2.pixels() {
            let hsl_original = Rgb::from(
                point.2 .0[0].into(),
                point.2 .0[1].into(),
                point.2 .0[2].into(),
            )
            .to_hsl();
            // Calculate average of neighbors
            let neighbor_coordinates =
                get_neighbor_coordinates(point.0, point.1, img_info.0, img_info.1);
            let mut r_sum = 0.0_f32;
            let mut g_sum = 0.0_f32;
            let mut b_sum = 0.0_f32;
            let mut count_sum = 0.0_f32;
            for neighbor_coordinate in neighbor_coordinates.iter() {
                let neighbor_point = img_info
                    .2
                    .get_pixel(neighbor_coordinate.0, neighbor_coordinate.1);
                r_sum += neighbor_point.0[0] as f32;
                g_sum += neighbor_point.0[1] as f32;
                b_sum += neighbor_point.0[2] as f32;
                count_sum += 1.0;
            }
            let r_avg = r_sum / count_sum;
            let g_avg = g_sum / count_sum;
            let b_avg = b_sum / count_sum;
            let hsl_average = Rgb::from(r_avg.into(), g_avg.into(), b_avg.into()).to_hsl();
            if (hsl_original.get_hue() - hsl_average.get_hue()).abs() > INTERPOLATION_THRESHOLD
                || (hsl_original.get_saturation() - hsl_average.get_saturation()).abs()
                    > INTERPOLATION_THRESHOLD
                || (hsl_original.get_lightness() - hsl_average.get_lightness()).abs()
                    > INTERPOLATION_THRESHOLD
            {
                let average_rgb = hsl_average.to_rgb();
                new_img_points.push((
                    point.0,
                    point.1,
                    average_rgb.get_red() as u8,
                    average_rgb.get_green() as u8,
                    average_rgb.get_blue() as u8,
                ));
            } else {
                new_img_points.push((
                    point.0,
                    point.1,
                    point.2 .0[0],
                    point.2 .0[1],
                    point.2 .0[2],
                ));
            }
        }
        let mut next_imgbuf = image::ImageBuffer::new(img_info.0, img_info.1);
        for p in new_img_points.iter() {
            let pixel = next_imgbuf.get_pixel_mut(p.0, p.1);
            *pixel = image::Rgb([p.2, p.3, p.4]);
        }
        img_info = (img_info.0, img_info.1, DynamicImage::ImageRgb8(next_imgbuf))
    }

    let mut imgbuf = image::ImageBuffer::new(img_info.0 * 2, img_info.1);
    for (x, y, pixel) in startingImg.pixels() {
        let new_x = x;
        let new_y = y;
        let new_pixel = imgbuf.get_pixel_mut(new_x, new_y);
        *new_pixel = pixel;
    }
    for p in new_img_points.iter() {
        let pixel = imgbuf.get_pixel_mut(p.0 + img_info.0, p.1);
        *pixel = Rgba([p.2, p.3, p.4, 255]);
    }
    fs::create_dir_all("images").unwrap();
    imgbuf.save("images/comparison.png").unwrap();
}
