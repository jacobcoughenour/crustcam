use std::cmp::max;

use nokhwa::{
	native_api_backend, nokhwa_initialize,
	pixel_format::{RgbAFormat, RgbFormat},
	query,
	utils::{ApiBackend, RequestedFormat, RequestedFormatType},
	CallbackCamera,
};

fn main() {
	let backend = native_api_backend().unwrap();
	let devices = query(backend).unwrap();
	let len = devices.len();
	println!("There are {} available cameras.", len);
	for device in devices {
		println!("{device}");
	}

	if len == 0 {
		println!("Error: No cameras found.");
		return;
	}

	// only needs to be run on OSX
	nokhwa_initialize(|granted| {
		println!("User said {}", granted);
	});
	let cameras = query(ApiBackend::Auto).unwrap();
	cameras.iter().for_each(|cam| println!("{:?}", cam));

	let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

	let first_camera = cameras.first().unwrap();

	let mut last_frame_time = std::time::Instant::now();

	let mut threaded = CallbackCamera::new(first_camera.index().clone(), format, move |buffer| {
		let image = buffer.decode_image::<RgbAFormat>().unwrap();
		let image_w = image.width();
		let image_h = image.height();

		let termsize::Size { rows, cols } = termsize::get().unwrap();

		let width = max(cols, 1);
		let height = max(rows - 1, 1);

		let x_step = image_w as f32 / width as f32;
		let y_step = (image_h as f32 / height as f32) * 0.5;
		// println!("{}x{} {}", image.width(), image.height(), image.len());

		// return the cursor to the top left
		print!("{}[1;1H", 27 as char);

		const fps_counter_width: u16 = 3;

		// set background to black
		print!("{}[48;2;0;0;0m", 27 as char);
		// set foreground to white
		print!("{}[38;2;255;255;255m", 27 as char);

		let now = std::time::Instant::now();
		let delta = now - last_frame_time;
		let fps = 1.0 / delta.as_secs_f32();
		let fps = fps as u32;
		print!("{:0>3}", fps);

		last_frame_time = now;

		for y in 0..height {
			for x in 0..width {
				if x < fps_counter_width && y == 0 {
					continue;
				}

				let top_pixel = image.get_pixel(
					((x as f32) * x_step) as u32,
					(((y * 2) as f32) * y_step) as u32,
				);
				let bottom_pixel = image.get_pixel(
					((x as f32) * x_step) as u32,
					(((y * 2 + 1) as f32) * y_step) as u32,
				);

				// change the background color to match the top pixel
				print!(
					"{}[48;2;{};{};{}m",
					27 as char, top_pixel[0], top_pixel[1], top_pixel[2]
				);

				// change the foreground color to match the bottom pixel
				print!(
					"{}[38;2;{};{};{}m▄",
					27 as char, bottom_pixel[0], bottom_pixel[1], bottom_pixel[2]
				);
			}
			print!("\n");
		}
	})
	.unwrap();
	threaded.open_stream().unwrap();
	#[allow(clippy::empty_loop)] // keep it running
	loop {
		threaded.poll_frame().unwrap();
	}
}
