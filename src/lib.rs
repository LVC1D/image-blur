#[derive(Debug, Clone)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug)]
pub struct ImageSoA {
    pub width: usize,
    pub height: usize,
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>,
}

pub fn blur_naive(image: &[Pixel], width: usize, height: usize) -> Vec<Pixel> {
    let mut image_blurred: Vec<Pixel> = vec![];

    for (i, pxl) in image.iter().enumerate() {
        if i < width || i >= width * (height - 1) || i % width == 0 || (i + 1) % width == 0 {
            image_blurred.push(pxl.clone());
        } else {
            let r_sum = image[i].r as u32
                + image[i - 1].r as u32
                + image[i + 1].r as u32
                + image[i - width].r as u32
                + image[i - width - 1].r as u32
                + image[i - width + 1].r as u32
                + image[i + width].r as u32
                + image[i + width - 1].r as u32
                + image[i + width + 1].r as u32;
            let r = (r_sum / 9) as u8;

            let g_sum = image[i].g as u32
                + image[i - 1].g as u32
                + image[i + 1].g as u32
                + image[i - width].g as u32
                + image[i - width - 1].g as u32
                + image[i - width + 1].g as u32
                + image[i + width].g as u32
                + image[i + width - 1].g as u32
                + image[i + width + 1].g as u32;
            let g = (g_sum / 9) as u8;

            let b_sum = image[i].b as u32
                + image[i - 1].b as u32
                + image[i + 1].b as u32
                + image[i - width].b as u32
                + image[i - width - 1].b as u32
                + image[i - width + 1].b as u32
                + image[i + width].b as u32
                + image[i + width - 1].b as u32
                + image[i + width + 1].b as u32;
            let b = (b_sum / 9) as u8;

            image_blurred.push(Pixel { r, g, b });
        }
    }
    image_blurred
}

pub fn blur_cache_optimized(image: &ImageSoA) -> ImageSoA {
    let mut rs_blurred: Vec<u8> = vec![];
    let mut gs_blurred: Vec<u8> = vec![];
    let mut bs_blurred: Vec<u8> = vec![];

    let width = image.width;
    let height = image.height;

    for (i, val) in image.r.iter().enumerate() {
        if i < width || i >= width * (height - 1) || i % width == 0 || (i + 1) % width == 0 {
            rs_blurred.push(*val);
        } else {
            let r_sum = image.r[i] as u32
                + image.r[i - 1] as u32
                + image.r[i + 1] as u32
                + image.r[i - width] as u32
                + image.r[i - width - 1] as u32
                + image.r[i - width + 1] as u32
                + image.r[i + width] as u32
                + image.r[i + width - 1] as u32
                + image.r[i + width + 1] as u32;
            let r = (r_sum / 9) as u8;

            rs_blurred.push(r);
        }
    }

    for (i, val) in image.g.iter().enumerate() {
        if i < width || i >= width * (height - 1) || i % width == 0 || (i + 1) % width == 0 {
            gs_blurred.push(*val);
        } else {
            let g_sum = image.g[i] as u32
                + image.g[i - 1] as u32
                + image.g[i + 1] as u32
                + image.g[i - width] as u32
                + image.g[i - width - 1] as u32
                + image.g[i - width + 1] as u32
                + image.g[i + width] as u32
                + image.g[i + width - 1] as u32
                + image.g[i + width + 1] as u32;
            let g = (g_sum / 9) as u8;

            gs_blurred.push(g);
        }
    }

    for (i, val) in image.b.iter().enumerate() {
        if i < width || i >= width * (height - 1) || i % width == 0 || (i + 1) % width == 0 {
            bs_blurred.push(*val);
        } else {
            let b_sum = image.b[i] as u32
                + image.b[i - 1] as u32
                + image.b[i + 1] as u32
                + image.b[i - width] as u32
                + image.b[i - width - 1] as u32
                + image.b[i - width + 1] as u32
                + image.b[i + width] as u32
                + image.b[i + width - 1] as u32
                + image.b[i + width + 1] as u32;
            let b = (b_sum / 9) as u8;

            bs_blurred.push(b);
        }
    }

    ImageSoA {
        width,
        height,
        r: rs_blurred,
        g: gs_blurred,
        b: bs_blurred,
    }
}

// We first define helper functions to construct the SIMD-optimized cache-firendly blur algorithm
fn blur_horizontal(channel: &[u8], width: usize, _height: usize) -> Vec<u8> {
    let mut chan = vec![];
    for (i, val) in channel.iter().enumerate() {
        if i % width == 0 || (i + 1) % width == 0 {
            chan.push(*val);
        } else {
            let chan_sum = channel[i] as u32 + channel[i - 1] as u32 + channel[i + 1] as u32;
            let ch = (chan_sum / 3) as u8;
            chan.push(ch);
        }
    }
    chan
}

fn blur_vertical(blurred_hrzntl: &[u8], width: usize, height: usize) -> Vec<u8> {
    // Your implementation
    let mut chan = vec![];
    for (i, val) in blurred_hrzntl.iter().enumerate() {
        if i < width || i >= width * (height - 1) {
            chan.push(*val);
        } else {
            let chan_sum = blurred_hrzntl[i] as u32
                + blurred_hrzntl[i - width] as u32
                + blurred_hrzntl[i + width] as u32;
            let ch = (chan_sum / 3) as u8;
            chan.push(ch);
        }
    }
    chan
}

pub fn blur_separable(image: &ImageSoA) -> ImageSoA {
    let width = image.width;
    let height = image.height;
    let red_blurred = blur_vertical(&blur_horizontal(&image.r, width, height), width, height);
    let green_blurred = blur_vertical(&blur_horizontal(&image.g, width, height), width, height);
    let blue_blurred = blur_vertical(&blur_horizontal(&image.b, width, height), width, height);

    ImageSoA {
        width,
        height,
        r: red_blurred,
        g: green_blurred,
        b: blue_blurred,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive() {
        let test_image = vec![
            Pixel {
                r: 100,
                g: 150,
                b: 200
            };
            512 * 512
        ];

        let blurred = blur_naive(&test_image, 512, 512);
        // Writing out the expected output would eb too long

        assert_eq!(blurred[0].r, 100);
        assert_eq!(blurred[0].g, 150);
        assert_eq!(blurred[0].b, 200);
    }

    #[test]
    fn test_cache_optimized() {
        let test_image = ImageSoA {
            width: 512,
            height: 512,
            r: vec![30; 512 * 512],
            g: vec![120; 512 * 512],
            b: vec![76; 512 * 512],
        };
        let blurred = blur_cache_optimized(&test_image);
        // Writing out the expected output would eb too long

        assert_eq!(blurred.r[513], 30);
        assert_eq!(blurred.g[513], 120);
        assert_eq!(blurred.b[513], 76);
    }

    #[test]
    fn test_blur_horizontal() {
        // Simple case: uniform row
        let channel = vec![
            100, 100, 100, 100, // Row 1 - all same
            50, 60, 70, 80, // Row 2 - varying
            100, 100, 100, 100, // Row 3 - all same
        ];
        let result = blur_horizontal(&channel, 4, 3);

        // Row 1: all 100 → should stay 100 (avg of 100,100,100 = 100)
        assert_eq!(result[1], 100); // Interior pixel

        // Row 2: pixel at index 5 (value 60)
        // Should be (50 + 60 + 70) / 3 = 180 / 3 = 60
        assert_eq!(result[5], 60);
    }

    #[test]
    fn test_separable_blur_edges() {
        let channel = vec![
            10, 20, 30, 40, // Row 0 (top edge)
            50, 60, 70, 80, // Row 1 (interior)
            90, 100, 110, 120, // Row 2 (bottom edge)
        ];

        let h_blurred = blur_horizontal(&channel, 4, 3);
        // Top edge pixel (index 1) should be blurred horizontally
        assert_eq!(h_blurred[1], 20); // (10+20+30)/3 = 20

        let b_blurred = blur_vertical(&h_blurred, 4, 3);
        // Left edge pixel (index 4) should be blurred vertically
        // Original was 50, after horizontal still 50 (edge), after vertical: (20+50+100)/3
        // Wait, let me recalculate...
    }

    #[test]
    fn test_full_blur() {
        let test_image = ImageSoA {
            width: 512,
            height: 512,
            r: vec![30; 512 * 512],
            g: vec![120; 512 * 512],
            b: vec![76; 512 * 512],
        };
        let blurred = blur_separable(&test_image);
        // Writing out the expected output would eb too long

        assert_eq!(blurred.r[513], 30);
        assert_eq!(blurred.g[513], 120);
        assert_eq!(blurred.b[513], 76);
    }
}

fn blur_horizontal_replicate(chan: &[u8], width: usize) -> Vec<u8> {
    let mut res = vec![];

    for (i, pxl) in chan.iter().enumerate() {
        if i % width == 0 || (i + 1) % width == 0 {
            res.push(*pxl);
        } else {
            let ch_sum = chan[i] as u32 + chan[i - 1] as u32 + chan[i + 1] as u32;

            let ch = (ch_sum / 3) as u8;
            res.push(ch);
        }
    }

    res
}

#[test]
fn test_replicate_horizontal() {
    let greens_chan = vec![30, 50, 70, 90, 50, 75, 10, 100, 10, 10, 60, 80];

    let green_blurred = blur_horizontal_replicate(&greens_chan, 4);

    assert_eq!(green_blurred[5], 45);
    assert_eq!(green_blurred[1], 50);
}
