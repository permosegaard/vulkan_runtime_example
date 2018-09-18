# rust & vulkan (vulkano) runtime example

cargo.toml excerpt:<br/>
<pre>
winit = "*"
vulkano = "*"
vulkano-win = "*"
vulkano-shader-derive = "*"
</pre>


getting a basic runtime up with vulkano takes more effort than it should and the triangle demo doesn't make it easy as everything's defined in main()

this should kick-start your project - i've tried to code around requiring "nll" turned on, however, enabling that is probably a good place to start if you're getting obscure/unintuitive ownership errors
