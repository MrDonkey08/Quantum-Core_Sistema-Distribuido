extern crate image;
extern crate num_complex;

use num_complex::Complex;

fn main() {
    let max_iterations = 256u16;
    let img_side: u32 = 4000;
    let cx_min: f64 = -2.0;
    let cx_max: f64 = 1.0;
    let cy_min: f64 = -1.5;
    let cy_max: f64 = 1.5;
    let scale_x = (cx_max - cx_min) / img_side as f64;
    let scale_y = (cy_max - cy_min) / img_side as f64;

    // Create a new ImgBuf
    let mut img_buf = image::ImageBuffer::new(img_side, img_side);

    // Calculate for each pixel
    for (x, y, pixel) in img_buf.enumerate_pixels_mut() {
        let cx = cx_min + x as f64 * scale_x;
        let cy = cy_min + y as f64 * scale_y;

        let c = Complex::new(cx, cy);
        let mut z = Complex::new(0f64, 0f64);

        let mut i = 0;
        for t in 0..max_iterations {
            if z.norm() > 2.0 {
                break;
            }

            z = z * z + c;
            i = t;
        }

        *pixel = image::Luma([i as u8]);
    }

    // Save image
    img_buf.save("fractal.png").unwrap();
}
