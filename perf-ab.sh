#!/bin/sh

# move existing build aside and create a new one
cargo build
hyperfine --warmup 10 './target/debug/spectral-raytracer-old' './target/debug/spectral-raytracer'
hyperfine --warmup 10 './target/debug/spectral-raytracer' './target/debug/spectral-raytracer-old'