# Image Blur Optimization: Algorithm and Data Layout Comparison

## Project Overview

In this project, we are testing several algorithms for blurring the image of various sizes.
Each approach differs in the following factors:

- Memory access and cache-friendliness
- API design complexity
- Overall latency in producing the output

## Benchmark Results Summary
   
   | Implementation | Time (ms) | vs Naive | Notes |
   |---------------|-----------|----------|-------|
   | Naive (AoS) | 2.08 | baseline | Good temporal locality |
   | Cache-opt (SoA) | 2.30 | +10% slower | Lost temporal locality |
   | Separable (SoA) | 1.83 | **12% faster** | Algorithm wins ✅ |

## Implementations 

The blurring technique used in all approaches is called "3 x 3 Box-Blur" where the blurred pixel value is the average of it and its 8 surrounding ones' values.

The below appraoches will be briefly gone over to see which algorithm and approach is most effective and efficient
at processing the pixels to blur them in the image:

1. Naive - utilizing AoS structure (Array of Structs), each struct being about 3 bytes (as each field - R, G and B carry values fo type `u8`).

The signature:
```rust
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn blur_naive(image: &[Pixel], width: usize, height: usize) -> Vec<Pixel> { 
    let mut image_blurred: Vec<Pixel> = vec![];

    for (i, pxl) in image.iter().enumerate() {
        if i < width ||
            i >= width * (height - 1) ||
            i % width == 0 ||
            (i + 1) % width == 0
        {
            image_blurred.push(pxl.clone());
        } else {
            let r_sum = image[i].r as u32 +
                     image[i-1].r as u32 +
                     image[i+1].r as u32 +
                     image[i-width].r as u32 +
                     image[i-width-1].r as u32 +
                     image[i-width+1].r as u32 +
                     image[i+width].r as u32 +
                     image[i+width-1].r as u32 +
                     image[i+width+1].r as u32;
            let r = (r_sum / 9) as u8;

            let g_sum = image[i].g as u32 +
                     image[i-1].g as u32 +
                     image[i+1].g as u32 +
                     image[i-width].g as u32 +
                     image[i-width-1].g as u32 +
                     image[i-width+1].g as u32 +
                     image[i+width].g as u32 +
                     image[i+width-1].g as u32 +
                     image[i+width+1].g as u32;
            let g = (g_sum / 9) as u8;

            let b_sum = image[i].b as u32 +
                     image[i-1].b as u32 +
                     image[i+1].b as u32 +
                     image[i-width].b as u32 +
                     image[i-width-1].b as u32 +
                     image[i-width+1].b as u32 +
                     image[i+width].b as u32 +
                     image[i+width-1].b as u32 +
                     image[i+width+1].b as u32;
            let b = (b_sum / 9) as u8;

            image_blurred.push(Pixel {
                r,
                g,
                b,
            });
        }
    }
    image_blurred

}
```

In this approach, as we iterate over each pixel, in order to caclulate the average of each channel's 
pixel value, we hop around the memory region loaded in ONE cache line (one array ,every 21-ish pixels chunk is loaded in one cache line)
-  this results in much higher count of cache hits to locate the adjacent channel's values - hence the calculations are rather efficient

(We will see why in the next section - Benchmarks)

2. Cache-optimized (non-separable)

This version switches from AoS to SoA structure (Struct of Arrays) where we have one Image that has all reds, greens and blues laid out in each corresponding vector.
Although one may think that because we have these fields in one struct - assuming that these vectors (as in heap data) live in that struct's
memory bucket - is FALSE.

These fields simply POINT to those arrays related to the struct definition - which in turn means that in the below signature,
we need to have a loop per each channel for this reason:

**Each array lives in completely scattered memory regions - potentially kilobytes apart - hence you can imagine the overhead that may add to the CPU
where it comes to reloading cache lines per each jump just to get the adjacent pixel values**

```rust
#[derive(Debug)]
pub struct ImageSoA {
    pub width: usize,
    pub height: usize,
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>,
}

pub fn blur_cache_optimized(image: &ImageSoA) -> ImageSoA {
    let mut rs_blurred: Vec<u8> = vec![];
    let mut gs_blurred: Vec<u8> = vec![];
    let mut bs_blurred: Vec<u8> = vec![];

    let width = image.width;
    let height = image.height;

    for (i, val) in image.r.iter().enumerate() {
        if i < width ||
            i >= width * (height - 1) ||
            i % width == 0 ||
            (i + 1) % width == 0
        {
            rs_blurred.push(*val);
        } else {
            let r_sum = image.r[i] as u32 +
                     image.r[i-1] as u32 +
                     image.r[i+1] as u32 +
                     image.r[i-width] as u32 +
                     image.r[i-width-1] as u32 +
                     image.r[i-width+1] as u32 +
                     image.r[i+width] as u32 +
                     image.r[i+width-1] as u32 +
                     image.r[i+width+1] as u32;
            let r = (r_sum / 9) as u8;

            rs_blurred.push(r);
        }
    }

    for (i, val) in image.g.iter().enumerate() {
        if i < width ||
            i >= width * (height - 1) ||
            i % width == 0 ||
            (i + 1) % width == 0
        {
            gs_blurred.push(*val);
        } else {
            let g_sum = image.g[i] as u32 +
                     image.g[i-1] as u32 +
                     image.g[i+1] as u32 +
                     image.g[i-width] as u32 +
                     image.g[i-width-1] as u32 +
                     image.g[i-width+1] as u32 +
                     image.g[i+width] as u32 +
                     image.g[i+width-1] as u32 +
                     image.g[i+width+1] as u32;
            let g = (g_sum / 9) as u8;

            gs_blurred.push(g);
        }
    }

    for (i, val) in image.b.iter().enumerate() {
        if i < width ||
            i >= width * (height - 1) ||
            i % width == 0 ||
            (i + 1) % width == 0
        {
            bs_blurred.push(*val);
        } else {
            let b_sum = image.b[i] as u32 +
                     image.b[i-1] as u32 +
                     image.b[i+1] as u32 +
                     image.b[i-width] as u32 +
                     image.b[i-width-1] as u32 +
                     image.b[i-width+1] as u32 +
                     image.b[i+width] as u32 +
                     image.b[i+width-1] as u32 +
                     image.b[i+width+1] as u32;
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
```

Now the reason we had to apply 3 separate loops per channel is because we want to respect the spatial locality - 
hence we make sure we load contiguous data of one channel in one loop and tell the compiler to deal with the calculations 
strictly within that colleciton.

Otherwise, if we had one loop to perform calculations for adjacent pixels of THREE channels - that's one loop
that needs to jump across three distinct memory regions - per iteration - which means a lot of cache misses and cold cache lines.

3. The separable version of Cache-optimized

Now let's take the above example and further simplify the calculations of pixels by dealing first with blurring horizontally (3 additions: left -> center -> right),
and then the vertical blurring (3 additions: top -> center -> bottom)

What happens is because perform these in 2 steps - each step requires 3 times elss of calculations per iteration, which is 
much easier ont he cahce line - and yet, we yield the same result

So it would look like this:

```rust
fn blur_horizontal(channel: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut chan = vec![];
    for (i, val) in channel.iter().enumerate() {
        if i % width == 0 ||
            (i + 1) % width == 0
        {
            chan.push(*val);
        } else {
            let chan_sum = channel[i] as u32 +
                channel[i-1] as u32 +
                channel[i+1] as u32;
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
        if i < width ||
            i >= width * (height - 1)
        {
            chan.push(*val);
        } else {
            let chan_sum = blurred_hrzntl[i] as u32 +
                blurred_hrzntl[i-width] as u32 +
                blurred_hrzntl[i+width] as u32;
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
```

## Benchmark Results 

```zsh
Benchmarking blur_naive: Collecting 100 samples in estimated 5.0051 s (2400 iterations
blur_naive              time:   [2.0749 ms 2.0784 ms 2.0821 ms]
                        change: [−0.8563% −0.6365% −0.4276%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking blur_cache_optimized: Collecting 100 samples in estimated 5.0189 s (2200
blur_cache_optimized    time:   [2.2800 ms 2.2973 ms 2.3284 ms]
                        change: [−0.0032% +0.8830% +2.4202%] (p = 0.15 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking blur_separable: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.2s, enable flat sampling, or reduce sample count to 50.
Benchmarking blur_separable: Collecting 100 samples in estimated 9.2191 s (5050 iterat
blur_separable          time:   [1.8252 ms 1.8277 ms 1.8303 ms]
                        change: [−0.7717% −0.5725% −0.3714%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
```

## Key Learnings 

- AoS keeps related data (r,g,b of same pixel) physically adjacent
- Temporal locality: accessing r→g→b of same pixel happens immediately
- Cache line holds all three channels for ~21 pixels
- SoA forces cold cache for each channel pass

As we learned through false-sharing principle,each vector fields would need a different
cache line - since they live in fundamentally different memory regions (potentially kilobytes apart).

*Critical insight:* Algorithm selection (separable filtering) provided more benefit than data layout micro-optimization.

## Running the Code 

As there is no binary to run, we can test it by running:
```bash
cargo test
```

You do need to be in the project subdirectory - `./week7/image-blur`

And to bench it (from the same locaiton), simply run:
```bash
cargo bench
```

## References - Link to imageproc source that inspired the separable approach
The [source code](https://docs.rs/imageproc/latest/src/imageproc/filter/mod.rs.html) from the imageproc crate that displays the real API for various image filtering logic
