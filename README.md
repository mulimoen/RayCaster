# RayCaster

This program implements a simple raytracer in OpenGL using [Glium](https://github.com/tomaka/glium). See also abstract.pdf for a short description of the program.

## Features
 * Maximum Intensity Projection
 * Isosurface extraction
 * Rendering of instanced packed cubes
 * Noise texture
 * Tweaking of parameters with [Dear Imgui](https://github.com/Gekkio/imgui-rs) interface
 * A simple reader of legacy VTK files
 * Rotate and translate camera using [Arcball](https://github.com/Twinklebear/arcball)


## Installation
cargo build --release

## Running
cargo run --release

This project was a part of the Computer Graphics course (spring 2017) at Uppsala University
