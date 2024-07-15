# Raysnail

The Raysail is a Monte Carlo type raytracer based on the [learn *Ray Tracing in One Weekend* series][book-series] using Rust. It was forked from the [Remda] project, which implements the features from first two books of the series. I've tried to add the features of the third, but I have doubts that my code is actually correct, even that the resulting images look alright.

Furthermore the Raysnail incorporates code from the [QBVH-Rust-Ray-Tracer], namely the triangle mesh module, OBJ file loading and the Blinn-Phong material code.

## Planned Additions

Beyond the ongoing work to incorporate all the improvements layed out in the third book of the series, fittingly named "The rest of your life", there are two features which I want to add to the Raysnail next.

### A preview window. 

There is a very crude implementation of a preview window right now in the example "preview_sdl2", but it needs to become more separated from the scene examples and command line paramaters to trace some scene with given height and width using the preview.

### Support for the PovRay SDL

I want to implement at least partial support for the [PovRay] scene definition language (SDL). Sadly, PovRay's material definitions are very different from the materials in the Raysnail, and it might be hard or even impossible to emulate PovRay materials properly, past trivial examples. At the time of writing this, I've implemented a very rudimentary SDL parser which can read PovRay camera and sphere definitions (see sdl/example.sdl). I want to expand this for more geometry and material definitons, but likely it will only support a subset of the PovRay SDL features.

## Run

The Raysnail is a library crate, but you can run built-in examples to try it.

Use `cargo run --example` to get examples list, then choose one to run.

For example, to get final scene in section 13.1 of *Ray Tracing in One Weekend*, run

```bash
cargo run --example rtow_13_1 --release
```
After some time (depending on your machine), you will get a `rtow_13_1.ppm` in current dir, that's your result.

To test the preview window, you can run

```bash
cargo run --example preview_sdl2 --release
```

If you want a bigger and clearer image, adjust `height()`, `depth()` and `samples()` parameters in source of the example file and re-run.

## LICENSE

GPLv3

Except:

- `example/earth-map.png`, download from [NASA][earth-map-source], falls in public domain.

[book-series]: https://raytracing.github.io/
[book-1]: https://raytracing.github.io/books/RayTracingInOneWeekend.html
[book-2]: https://raytracing.github.io/books/RayTracingTheNextWeek.html
[book-3]: https://raytracing.github.io/books/RayTracingTheRestOfYourLife.html
[earth-map-source]: http://visibleearth.nasa.gov/view.php?id=57752
[Remda]: https://github.com/7sDream/remda
[QBVH-Rust-Ray-Tracer]: https://github.com/miguelggcc/QBVH-Rust-Ray-Tracer
[PovRay]: http://www.povray.org/
