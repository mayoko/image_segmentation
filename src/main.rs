extern crate image;

use std::path::Path;
use std::cmp::{Ordering, Ord};
use std::collections::binary_heap::{BinaryHeap};

use image::Rgb;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Segment {
    variance: f64,
    left: usize,
    right: usize,
    top: usize,
    bottom: usize
}

impl Eq for Segment {}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.variance.partial_cmp(&other.variance).unwrap()
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.variance.partial_cmp(&other.variance)
    }
}

fn main() {
    generate_segmented_image(Path::new("./test.png"), "./gen_image/test_segmented");
}

fn generate_segmented_image(image_path: &Path, output_image_base_path: &str) {
    let original_img = image::open(image_path).unwrap();
    let original_img = original_img.to_rgb8();
    let width = original_img.width() as usize;
    let height = original_img.height() as usize;

    let mut rgb_cumsum = vec![vec![vec![0.0; width + 1]; height + 1]; 3];
    let mut rgb_sqr_cumsum = vec![vec![vec![0.0; width + 1]; height + 1]; 3];
    for rgb in 0..3 {
        for i in 1..height+1 {
            for j in 1..width+1 {
                let pixel = original_img.get_pixel((j-1).try_into().unwrap(), (i-1).try_into().unwrap());
                rgb_cumsum[rgb][i][j] = rgb_cumsum[rgb][i-1][j] + rgb_cumsum[rgb][i][j-1] - rgb_cumsum[rgb][i-1][j-1] + pixel[rgb] as f64;
                rgb_sqr_cumsum[rgb][i][j] = rgb_sqr_cumsum[rgb][i-1][j] + rgb_sqr_cumsum[rgb][i][j-1] - rgb_sqr_cumsum[rgb][i-1][j-1] + (pixel[rgb] as f64 * pixel[rgb] as f64);
            }
        }
    }

    let mut heap = BinaryHeap::new();
    heap.push(Segment {variance: 0.0, left: 0, right: width, top: 0, bottom: height});
    for t in 0..1001 {
        let top_variance_segment = heap.pop().unwrap();
        println!("choose: {:?}", &top_variance_segment);
        let left = top_variance_segment.left;
        let right = top_variance_segment.right;
        let top = top_variance_segment.top;
        let bottom = top_variance_segment.bottom;

        if right - left <= 1 || bottom - top <= 1 {
            continue;
        }

        let mx = (left + right) / 2;
        let my = (top + bottom) / 2;
        // 左上
        heap.push(Segment { variance: get_scaled_variance(left, mx, top, my, &rgb_cumsum, &rgb_sqr_cumsum), left, right: mx, top, bottom: my });
        // 右上
        heap.push(Segment { variance: get_scaled_variance(mx, right, top, my, &rgb_cumsum, &rgb_sqr_cumsum), left: mx, right, top, bottom: my });
        // 左下
        heap.push(Segment { variance: get_scaled_variance(left, mx, my, bottom, &rgb_cumsum, &rgb_sqr_cumsum), left, right: mx, top: my, bottom });
        // 右下
        heap.push(Segment { variance: get_scaled_variance(mx, right, my, bottom, &rgb_cumsum, &rgb_sqr_cumsum), left: mx, right, top: my, bottom });

        if t % 10 == 0 {
            save_current_segmented_image(&heap, width, height, &rgb_cumsum, Path::new(&format!("{}{}.png", output_image_base_path, t)));
        }
    }
}

fn get_scaled_variance(left: usize, right: usize, top: usize, bottom: usize, rgb_cumsum: &[Vec<Vec<f64>>], rgb_sqr_cumsum: &[Vec<Vec<f64>>]) -> f64 {
    let mut result = 0.0;
    let n = (right - left) as f64 * (bottom - top) as f64;
    for rgb in 0..3 {
        let mean = (rgb_cumsum[rgb][bottom][right] + rgb_cumsum[rgb][top][left] - rgb_cumsum[rgb][top][right] - rgb_cumsum[rgb][bottom][left]) / n;
        let sqr_mean = (rgb_sqr_cumsum[rgb][bottom][right] + rgb_sqr_cumsum[rgb][top][left] - rgb_sqr_cumsum[rgb][top][right] - rgb_sqr_cumsum[rgb][bottom][left]) / n;
        result += sqr_mean - mean * mean;
    }
    result * n
}

fn get_mean_color(left: usize, right: usize, top: usize, bottom: usize, rgb_cumsum: &[Vec<Vec<f64>>]) -> Rgb<u8> {
    let mut color_f64 = vec![0.0; 3];
    for i in 0..3 {
        color_f64[i] += rgb_cumsum[i][bottom][right] + rgb_cumsum[i][top][left] - rgb_cumsum[i][top][right] - rgb_cumsum[i][bottom][left];
        color_f64[i] /= (right - left) as f64 * (bottom - top) as f64;
    }
    Rgb([color_f64[0] as u8, color_f64[1] as u8, color_f64[2] as u8])
}

fn save_current_segmented_image(segments: &BinaryHeap<Segment>, width: usize, height: usize, rgb_cumsum: &[Vec<Vec<f64>>], output_image_path: &Path) {
    // println!("=========================");
    let mut generated_img = image::RgbImage::new(width as u32, height as u32);
    for segment in segments.iter() {
        // println!("{:?}", segment);
        let left = segment.left;
        let right = segment.right;
        let top = segment.top;
        let bottom = segment.bottom;
        let color = get_mean_color(left, right, top, bottom, rgb_cumsum);

        for y in top..bottom {
            for x in left..right {
                generated_img.put_pixel(x.try_into().unwrap(), y.try_into().unwrap(), color);
            }
        }
    }
    generated_img.save(output_image_path).unwrap();
}