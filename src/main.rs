use std::env;
use std::cmp::max;

use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use image::Rgba;
use image::error::ImageResult;
use image::imageops;
use image::io::Reader as ImageReader;

use leonhard::linear_algebra::Vector;

fn open_image(file: &String) -> ImageResult<DynamicImage> {
	ImageReader::open(file)?.decode()
}

fn gaussian_filter(x: f32, y: f32, sigma: f32) -> f32 {
	let a = 2. * std::f32::consts::PI * sigma * sigma;
	let b = 2. * sigma * sigma;
	(-(x * x + y * y) / b).exp() / a
}

#[inline(always)]
fn clamp(n: f32, min: f32, max: f32) -> f32 {
	if n < min {
		min
	} else if n > max {
		max
	} else {
		n
	}
}

#[inline(always)]
fn color_to_vector(color: &Rgba<u8>) -> Vector::<f32> {
	Vector::<f32>::from_vec(vec![
		color[0] as f32 / 255.,
		color[1] as f32 / 255.,
		color[2] as f32 / 255.,
	])
}

#[inline(always)]
fn vector_to_color(vector: &Vector::<f32>) -> Rgba<u8> {
	Rgba([
		(clamp(*vector.x(), 0., 1.) * 255.) as _,
		(clamp(*vector.y(), 0., 1.) * 255.) as _,
		(clamp(*vector.z(), 0., 1.) * 255.) as _,
		//(*vector.x() * 255.) as _,
		//(*vector.y() * 255.) as _,
		//(*vector.z() * 255.) as _,
		255
	])
}

fn difference_of_gaussian(img: &DynamicImage, sigma: f32, k: f32) -> DynamicImage {
	let mut result = DynamicImage::new_rgb8(img.width(), img.height());
	let radius = max((sigma * sigma) as i32, ((k * sigma) * (k * sigma)) as i32);

	for x in 0..img.width() {
		for y in 0..img.height() {
			let mut color = Vector::<f32>::new(3);

			for i in -radius..radius {
				for j in -radius..radius {
					if x as i32 + i < 0
						|| y as i32 + j < 0
						|| x as i32 + i >= img.width() as i32
						|| y as i32 + j >= img.height() as i32 {
						continue;
					}

					let pixel_color = img.get_pixel((x as i32 + i) as _, (y as i32 + j) as _);
					let gauss = gaussian_filter(i as _, j as _, k * sigma)
						- gaussian_filter(i as _, j as _, sigma);
					color += color_to_vector(&pixel_color) * gauss;
				}
			}

			result.put_pixel(x, y, vector_to_color(&color));
		}
	}

	result
}

fn draw_point(img: &mut DynamicImage, x: usize, y: usize, radius: isize) {
	for i in -radius..radius {
		for j in -radius..radius {
			if i * i + j * j > radius * radius {
				continue;
			}

			let x_coord = x as isize + i;
			let y_coord = y as isize + j;
			if x_coord < 0 || x_coord >= img.width() as isize
				|| y_coord < 0 || y_coord >= img.height() as isize {
				continue;
			}
			img.put_pixel(x_coord as _, y_coord as _, Rgba([255, 0, 255, 255]));
		}
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() <= 1 {
		eprintln!("Please specify one or several images!");
		std::process::exit(1);
	}

	let mut images = Vec::<(DynamicImage, usize)>::new();
	let mut width: usize = 0;
	let mut height: usize = 0;
	for i in 1..args.len() {
		let img_result = open_image(&args[i]);
		if img_result.is_err() {
			eprintln!("Failed to open image `{}`!", args[i]);
			std::process::exit(1);
		}

		let img = img_result.unwrap();
		//let img_pre = img.resize(400, 300, imageops::FilterType::Triangle); // TODO Remove
		let img_pre = img;
		let img_post = difference_of_gaussian(&img_pre, 3., 0.5);
		let y = height;
		width = max(width, img_post.width() as usize);
		height += img_post.height() as usize;

		images.push((img_post, y));
	}

	let mut final_image = DynamicImage::new_rgb8(width as _, height as _);
	for (_, img) in images.iter().enumerate() {
		imageops::overlay(&mut final_image, &img.0, 0, img.1 as _);
	}

	// TODO Apply SIFT

    if final_image.save("output.jpg").is_err() {
		eprintln!("Failed to save image!");
		std::process::exit(1);
	}
}
