# Raysnail

The Raysail is a Monte Carlo type raytracer based on the [learn *Ray Tracing in One Weekend* series][book-series] using Rust. It was forked from the [Remda] project, which implements the features from first two books of the series. I've tried to add the features of the third, but I have doubts that my code is actually correct, even that the resulting images look alright.

Furthermore the Raysnail incorporates code from the [QBVH-Rust-Ray-Tracer], namely the triangle mesh module, OBJ file loading and the Blinn-Phong material code.

![Quadric shapes](https://private-user-images.githubusercontent.com/62077919/353458938-cac220bd-3b68-492d-93ce-3f6e6f21d414.jpg?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3MjIzNDMzNDIsIm5iZiI6MTcyMjM0MzA0MiwicGF0aCI6Ii82MjA3NzkxOS8zNTM0NTg5MzgtY2FjMjIwYmQtM2I2OC00OTJkLTkzY2UtM2Y2ZTZmMjFkNDE0LmpwZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNDA3MzAlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjQwNzMwVDEyMzcyMlomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWZkOTljMTcxNGM5YTVmMzYzMDAzYWY2ZWNlYTQyODcwMzQ3ZThkMTUzZmZkZjE2YjE5NTYwNDFmMzk3NzBlNTUmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0JmFjdG9yX2lkPTAma2V5X2lkPTAmcmVwb19pZD0wIn0.qw8H6gW1SbTHyX1qQJbHysfqTfy_htcvXIqxeRymFyc)

## Planned Additions

Beyond the ongoing work to incorporate all the improvements layed out in the third book of the series, fittingly named "The rest of your life", there are some features which I want to add to the Raysnail next.

### A preview window

There is a very crude implementation of a preview window right now in the example "preview_sdl2", but it needs to become more separated from the scene examples and command line paramaters to trace some scene with given height and width using the preview.

### Support for the PovRay SDL

I want to implement at least partial support for the [PovRay] scene definition language (SDL). Sadly, PovRay's material definitions are very different from the materials in the Raysnail, and it might be hard or even impossible to emulate PovRay materials properly, past trivial examples. At the time of writing this, I've implemented a very rudimentary SDL parser which can read PovRay camera and sphere definitions (see sdl/example.sdl). I want to expand this for more geometry and material definitons, but likely it will only support a subset of the PovRay SDL features.

### 3D Fractal rendering capabilities

As a fan of 3D fractals I'd like to implement some fractal rendering code. At the moment there is a crude implmentation of the mandelbulb, which renders only very low quality, but it can serve as a proof of concept that iut is possible to use the raysnail code to render 3D fractals. Maybe it requires a different frontend though.

Please check the Wiki for a full list of features and plans:
https://github.com/Varkalandar/raysnail/wiki

## Run

If you have installed Rust and Cargo, an easy way to run the raysnail is this command: 

```bash
cargo run -r --bin raysnail -- -w 800 -h 500 --samples 65 --scene sdl/example.sdl
```
* --scene <File> tells which SDL file to read and render
* --samples is the number of samples taken per pixel. Taking more samples improves the image quality, but also raises the rendering time.
* -w <Integer> is the image width
* -h <Integer> is the image height

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
