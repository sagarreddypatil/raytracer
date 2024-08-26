#!/bin/sh

# move existing build aside and create a new one
cargo build
hyperfine --warmup 5 './target/debug/spectral-raytracer-old' './target/debug/spectral-raytracer'