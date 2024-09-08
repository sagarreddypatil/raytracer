#!/bin/sh

# move existing build aside and create a new one
cargo build
hyperfine --warmup 10 './target/debug/raytracer-old' './target/debug/raytracer'
hyperfine --warmup 10 './target/debug/raytracer' './target/debug/raytracer-old'