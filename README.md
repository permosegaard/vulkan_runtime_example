# rust & vulkano (vulkano) runtime example

cargo.toml excerpt:<br/>
winit = "*"<br/>
vulkano = "*"<br/>
vulkano-win = "*"<br/>
vulkano-shader-derive = "*"<br/>


getting a basic runtime up with vulkano takes more effort than it should and the triangle demo doesn't make it easy as everything's defined in main()

this should kick-start your project - i've tried to code around requiring "nll" turned on but that's probably good to try first if you get any ownership errors
