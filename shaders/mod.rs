pub mod vertex {
	#[derive(VulkanoShader)]
	#[ty = "vertex"]
	#[path = "src/visualisation/runtime/shaders/vertex.glsl"]
	#[allow(dead_code)]
	struct Dummy;
}

pub mod fragment {
	#[derive(VulkanoShader)]
	#[ty = "fragment"]
	#[path = "src/visualisation/runtime/shaders/fragment.glsl"]
	#[allow(dead_code)]
	struct Dummy;
}

pub fn build_shaders(renderer: &super::Renderer) -> (fragment::Shader, vertex::Shader) {
	let fragment = fragment::Shader::load(
		renderer
			.device
			.clone()
	)
		.unwrap();

	let vertex = vertex::Shader::load(
		renderer
			.device
			.clone()
	)
		.unwrap();

	(fragment, vertex)
}