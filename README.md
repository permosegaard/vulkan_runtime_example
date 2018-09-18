# vulkan_runtime_example

cargo.toml excerpt:
winit = "*"
vulkano = "*"
vulkano-win = "*"
vulkano-shader-derive = "*"


getting a basic runtime up with vulkano takes more effort than it should and the triangle demo doesn't make it easy as everything's defined in main()

this should kick-start your project - i've tried to code around requiring "nll" turned on but that's probably good to try first if you get any ownership errors
