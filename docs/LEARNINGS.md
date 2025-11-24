# SIMD Image-Processing: Learnings

## Why does separable filtering work mathematically?

Explain why horizontal-then-vertical gives the same result as 2D box blur
What property of the box blur makes this decomposition valid?

Because blurring per-step is returning the average of the adjacent pixel values once (horizontally or vertically),
the outcome will be mathematically the same.

Examples:
1. Non-separable. Grab 9 pixels - the one in question + all 8 "surrounding" ones (say, their valeus are 1, 2, ... 9) , divide them
all by the number of pixels = 45 / 9 = 5

We get 5 but you can imagine how many times (and how far in memory) we had to jump over the regions to get those values within the channel's memory.

2. Separable. Same pixel values as before - but now For horizontal - only left - center - right oens - divinde by 3,
and then store these (with left & right edges untouched) to the output.

Then vertical - again only 3 (but only top, center and bottom ones) -> sum them up, divide by 3 - do not touch the top and bottom pixels, store 
in the new output

Which means:
(((4 + 5 + 6) / 3) + ((1 + 2 + 3) / 3) + ((7 + 8 + 9) / 3)) / 3 = (5 + 2 + 8) / 3 = 5

Same output! Smoother memory accessing.

## Why wasn't the naive version terrible?

- AoS keeps related data (r,g,b of same pixel) physically adjacent
- Temporal locality: accessing r→g→b of same pixel happens immediately
- Cache line holds all three channels for ~21 pixels
- SoA forces cold cache for each channel pass

As we learned through false-sharing principle,each vector fields would need a different 
cache line - since they live in fundamentally different memory regions (potentially kilobytes apart),
which cause the need for 3 loops to somehow facilitate the transition 
from one memory region to another to handle each pixel's channel vector.

## Why is separable filtering faster?

In a non-separable version, we have to take into account NINE pixels per single operation in one channel, given 
also that we are in the SoA structure, the jumping around the memory regions would add to the overhead.

In a separable version, however, we deal with three pixels per iteraiton only:
- Horizontal: left - center - right
- Vertical: top - center - bottom

That sums up to 6 additions per an aggregated operaiton - much easier for the memory access and the CPU processing
as each step is more cache-friendly

Complexity: O(n × 6) vs O(n × 9) = 33% fewer operations
Plus: horizontal pass is perfectly sequential (no striding at all!)

## What was wrong with our initial "cache-optimized" approach?

We can reference the answer for the question regarding why the Naive approach was better - 
it all comes down to the implicaitons of spatial or temporal locality

The spatial locality within the AoS wins in terms of cache-line-friendliness

The SoA 3-loop approach still accessed pixels in 2D 
(i-width, i+width for vertical neighbors), maintaining the strided access problem 
while losing the temporal locality benefit of AoS.

## Key lesson: Algorithm selection matters

I had to critically judge and examine the validity of the concepts explained during the session
visiting [this source code](https://docs.rs/imageproc/latest/src/imageproc/filter/mod.rs.html) for the `imageproc` crate.

The separable 2-step filtering was what the discovery of the optimized image processing became viable
