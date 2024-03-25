use std::sync::{Arc, Mutex};
use std::thread;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Rgba};

// Define a simple box blur filter function
fn box_blur_filter(input: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut output = ImageBuffer::new(input.width(), input.height());

    for y in 0..input.height() {
        for x in 0..input.width() {
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = (x as i32 + dx).clamp(0, input.width() as i32 - 1) as u32;
                    let ny = (y as i32 + dy).clamp(0, input.height() as i32 - 1) as u32;

                    let pixel = input.get_pixel(nx, ny);
                    let (r, g, b, _) = pixel.channels4();

                    sum_r += r as u32;
                    sum_g += g as u32;
                    sum_b += b as u32;
                }
            }

            let count = 9; // Number of pixels in the box
            let avg_r = (sum_r / count).clamp(0, 255) as u8;
            let avg_g = (sum_g / count).clamp(0, 255) as u8;
            let avg_b = (sum_b / count).clamp(0, 255) as u8;

            output.put_pixel(x, y, Rgba([avg_r, avg_g, avg_b, 255]));
        }
    }

    output
}

fn main() {
    // Load the input image
    let input_image = image::open("./../rusty_image_processing/images/go_maskotchen_many.jpeg").expect("Failed to load input image");

    // Convert to ImageBuffer for easy pixel access
    let input_image_buffer = input_image.to_rgba8();

    // Create a mutex-protected clone of the input image buffer for shared mutability
    let input_image_buffer_shared = Arc::new(Mutex::new(input_image_buffer));

    // Create a vector to store the handles to the threads
    let mut handles = vec![];

    // Clone the Arc reference for each thread
    let input_image_buffer_shared_clone = Arc::clone(&input_image_buffer_shared);

    // Get the width and height of the image
    let (width, height) = input_image_buffer_shared_clone.lock().unwrap().dimensions();

    // Number of threads for parallel processing
    let num_threads = num_cpus::get();

    for _ in 0..num_threads {
        let input_image_buffer_shared = Arc::clone(&input_image_buffer_shared);

        // Clone the Arc reference for each thread
        let handle = thread::spawn(move || {
            // Lock the mutex to gain access to the ImageBuffer
            let mut input_image_buffer = input_image_buffer_shared.lock().unwrap();

            // Apply the filter to the image
            let processed_image = box_blur_filter(&input_image_buffer);

            // Update the input image buffer with the processed image
            *input_image_buffer = processed_image;
        });

        handles.push(handle);
    }

    // Wait for all threads to finish processing
    for handle in handles {
        handle.join().unwrap();
    }

    // Save the processed image
    let input_image_buffer_shared = input_image_buffer_shared.lock().unwrap();
    let output_image = input_image_buffer_shared.clone();
    output_image
        .save("output_image.jpg")
        .expect("Failed to save output image");
}
