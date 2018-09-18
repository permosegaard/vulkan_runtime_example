mod shaders;


use std::sync::Arc;

use winit::{WindowBuilder, EventsLoop, Window};

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture, DynamicState};
use vulkano::image::SwapchainImage;
use vulkano::sync::{FenceSignalFuture, FlushError, GpuFuture, JoinFuture, now};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{Capabilities, PresentFuture, Surface, SwapchainAcquireFuture};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError, acquire_next_image};
use vulkano::instance::{Instance, PhysicalDevice, PhysicalDeviceType, layers_list};

use vulkano_win::VkSurfaceBuild;


type InstanceType = Arc<Instance>;
type EventsLoopType = EventsLoop;
type WindowType = Arc<Surface<Window>>;
type CapabilitiesType = Capabilities;
type DimensionsType = [u32; 2];
type DeviceType = Arc<Device>;
type QueueType = Arc<Queue>;

type SwapchainType = Arc<Swapchain<Window>>;
type ImagesType = Vec<Arc<SwapchainImage<Window>>>;
type RenderpassType = Arc<RenderPassAbstract + Send + Sync>;
type VertexbufferType = Arc<CpuAccessibleBuffer<[Vertex]>>;
type PipelineType = Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, Box<PipelineLayoutAbstract + Send + Sync>, Arc<RenderPassAbstract + Send + Sync>>>;
type FramebuffersType = Vec<Arc<FramebufferAbstract + Send + Sync>>;
type DynamicstateType = DynamicState;
type CommandbufferType = AutoCommandBuffer;
type FutureType = Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>, AutoCommandBuffer>, Window>>, FlushError>;


#[derive(Debug, Clone)]
pub struct Vertex {
    position: [f32; 2],
    /*uv: [f32;2],
    normal: [f32;3]*/
}
impl_vertex!(Vertex, position/*, normal, uv*/);

pub struct Renderer {
	instance: InstanceType,
	pub events_loop: EventsLoopType,
	window: WindowType,
	capabilities: CapabilitiesType,
	device: DeviceType,
	queue: QueueType
}

pub struct Other {
	swapchain: SwapchainType,
	images: ImagesType,
	renderpass: RenderpassType,
	vertexbuffer : VertexbufferType,
	fragment_shader: shaders::fragment::Shader,
	vertex_shader: shaders::vertex::Shader,
	pipeline: PipelineType,
	framebuffers: FramebuffersType,
	previous_frame_end: Option<Box<dyn GpuFuture>>,
	dynamicstate: DynamicstateType
}

pub struct Runtime {
	pub renderer: Renderer,
	other: Other,
	recreate: bool
}


fn create_instance() -> InstanceType {
	use vulkano::instance::ApplicationInfo;
	let extensions = ::vulkano_win::required_extensions();
	let app_info = app_info_from_cargo_toml!();

	let layers_available: Vec<_> = layers_list()
		.unwrap()
		.collect();

	let layers_chosen : Vec<_> = layers_available
		.iter()
		.filter(
			|layer| {
				let description = layer.description().to_lowercase();
				
				description.contains("validation") || description.contains("monitoring") || description.contains("driver")
				
				//description.contains("driver")
			}
		)
		.map(|layer| layer.name())
		.collect();

	Instance::new(Some(&app_info), &extensions, layers_chosen)
		.unwrap()
}

fn create_eventsloop() -> EventsLoopType {
	EventsLoopType::new()
}

fn create_window(events_loop: &EventsLoopType, instance: &InstanceType) -> WindowType {
	WindowBuilder::new()
		.build_vk_surface(&events_loop, instance.clone())
		.unwrap()
}

fn create_physical(instance: &InstanceType) -> PhysicalDevice {
	PhysicalDevice::enumerate(&instance)
		.find(|device| device.ty() == PhysicalDeviceType::DiscreteGpu)
		.or_else(|| PhysicalDevice::enumerate(&instance).nth(0))
		.unwrap()
}

fn create_capabilities(instance: &InstanceType, window: &WindowType) -> CapabilitiesType {
	let physical = create_physical(&instance); // NOTE: workaround not having 'nll' enabled

	window
		.capabilities(physical)
		.unwrap()
}

fn create_dimensions(renderer: &Renderer) -> DimensionsType {
	renderer
		.capabilities
		.current_extent
		.unwrap()
}

fn create_device_and_queues(instance: &InstanceType, window: &WindowType) -> (DeviceType, QueueType) {
	let physical = create_physical(&instance); // NOTE: workaround not having 'nll' enabled

	let queue = physical
		.queue_families()
		.find(
			|queue| {
				queue.supports_graphics() && window.is_supported(*queue).unwrap_or(false)
			}
		)
		.unwrap();

	let (device, mut queues) = {
		let device_ext = DeviceExtensions {
			khr_swapchain: true,
			.. DeviceExtensions::none()
		};

		Device::new(physical, physical.supported_features(), &device_ext, [(queue, 0.5)].iter().cloned())
			.unwrap()
	};

	(device, queues.next().unwrap())
}

fn create_swapchain_and_images(renderer: &Renderer) -> (SwapchainType, ImagesType) {
	let dimensions = renderer.capabilities.current_extent
		.unwrap_or([1024, 768]);
	
	let alpha = renderer.capabilities.supported_composite_alpha
		.iter()
		.next()
		.unwrap();

	let format = renderer.capabilities.supported_formats[0].0;

	let present_mode = renderer.capabilities.present_modes
		.iter()
		.find(|mode| *mode == PresentMode::Mailbox)
		.unwrap_or(PresentMode::Fifo);

	Swapchain::new(
		renderer.device.clone(), renderer.window.clone(), renderer.capabilities.min_image_count, format, dimensions, 1,
		renderer.capabilities.supported_usage_flags, &renderer.queue, SurfaceTransform::Identity, alpha, present_mode, true, None
	)
		.unwrap()
}

fn create_renderpass(renderer: &Renderer, swapchain: &SwapchainType) -> RenderpassType {
	Arc::new(
		single_pass_renderpass!(
			renderer.device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: swapchain.format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		)
			.unwrap()
	)
}

fn create_vertexbuffer(renderer: &Renderer) -> VertexbufferType {
	CpuAccessibleBuffer::from_iter(
		renderer.device.clone(), BufferUsage::all(), 
		[
			Vertex { position: [-0.5, -0.25] },
			Vertex { position: [0.0, 0.5] },
			Vertex { position: [0.25, -0.1] }
		]
			.iter()
			.cloned()
	)
		.unwrap()
}

fn create_pipeline(renderer: &Renderer, renderpass: &RenderpassType, fragment_shader: &shaders::fragment::Shader, vertex_shader: &shaders::vertex::Shader) -> PipelineType {
	Arc::new(
		GraphicsPipeline::start()
	        .vertex_input_single_buffer()
			.vertex_shader(vertex_shader.main_entry_point(), ())
			.triangle_list()
			.viewports_dynamic_scissors_irrelevant(1)
			.fragment_shader(fragment_shader.main_entry_point(), ())
			.render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
			.build(renderer.device.clone())
			.unwrap()
	)
}

fn create_framebuffers(images: &ImagesType, renderpass: &RenderpassType) -> FramebuffersType {
	images
		.iter()
		.map(
			|image| {
				Arc::new(
					Framebuffer::start(renderpass.clone())
						.add(image.clone())
						.unwrap()
						.build()
						.unwrap()
				) as Arc<FramebufferAbstract + Send + Sync>
			}
		)
		.collect()
}

fn create_dynamicstate(renderer: &Renderer, dimensions: DimensionsType) -> DynamicstateType {
    DynamicState {
        line_width: None,
        viewports: Some(
			vec![
				Viewport {
					origin: [0.0, 0.0],
					dimensions: [dimensions[0] as f32, dimensions[1] as f32],
					depth_range: 0.0 .. 1.0,
        		}
			]
		),
        scissors: None,
	}
}

fn create_commandbuffer(runtime: &Runtime, which: usize) -> CommandbufferType {
	AutoCommandBufferBuilder::primary_one_time_submit(runtime.renderer.device.clone(), runtime.renderer.queue.family())
		.unwrap()
		.begin_render_pass(runtime.other.framebuffers[which].clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
        .unwrap()
        .draw(runtime.other.pipeline.clone(), &runtime.other.dynamicstate, runtime.other.vertexbuffer.clone(), (), ())
        .unwrap()
        .end_render_pass()
        .unwrap()
		.build()
		.unwrap()
}

fn create_future(runtime: &mut Runtime, which: usize, acquire_future: SwapchainAcquireFuture<Window>) -> FutureType {
	let commandbuffer = create_commandbuffer(&runtime, which);

	runtime.other
		.previous_frame_end
		.take()
		.unwrap()
		.join(acquire_future)
		.then_execute(runtime.renderer.queue.clone(), commandbuffer)
		.unwrap()
		.then_swapchain_present(runtime.renderer.queue.clone(), runtime.other.swapchain.clone(), which)
		.then_signal_fence_and_flush()
}

fn build_renderer() -> Renderer {
	let events_loop = create_eventsloop();

	let instance = create_instance();

	let window = create_window(&events_loop, &instance);

	let capabilities = create_capabilities(&instance, &window);

	let (device, queue) = create_device_and_queues(&instance, &window);

	Renderer { events_loop, instance, window, capabilities, device, queue }
}

fn build_other(renderer: &Renderer) -> Other {
	let (swapchain, images) = create_swapchain_and_images(&renderer);

	let renderpass = create_renderpass(&renderer, &swapchain);

	let vertexbuffer = create_vertexbuffer(&renderer);

	let (fragment_shader, vertex_shader) = shaders::build_shaders(&renderer);

	let pipeline = create_pipeline(&renderer, &renderpass, &fragment_shader, &vertex_shader);

	let framebuffers = create_framebuffers(&images, &renderpass);

	let previous_frame_end = Some(Box::new(now(renderer.device.clone())) as Box<GpuFuture>);

	let dynamicstate = create_dynamicstate(&renderer, create_dimensions(&renderer));

	Other { swapchain, images, renderpass, vertexbuffer, fragment_shader, vertex_shader, pipeline, framebuffers, previous_frame_end, dynamicstate }
}

pub fn recreate(runtime: &mut Runtime) -> bool {
	let dimensions = create_dimensions(&runtime.renderer);

	match runtime.other.swapchain.recreate_with_dimension(dimensions) {
		Err(SwapchainCreationError::UnsupportedDimensions) => { return false; },
		Err(err) => panic!("{:?}", err),
		Ok((swapchain, images)) => {
			runtime.other.swapchain = swapchain;
			runtime.other.images = images;
		}
	}

	runtime.other.dynamicstate = create_dynamicstate(&runtime.renderer, dimensions);

	runtime.other.framebuffers = create_framebuffers(&runtime.other.images, &runtime.other.renderpass);

	true
}

pub fn new() -> Runtime {
	let renderer = build_renderer();

	let other = build_other(&renderer);

	let recreate = false;

	Runtime { renderer, other, recreate }
}

pub fn redraw(mut runtime: &mut Runtime) {
	runtime.other.previous_frame_end = {
		let mut unwrapped = runtime.other.previous_frame_end.take().unwrap();
		unwrapped.cleanup_finished();
		Some(unwrapped)
	};

	if runtime.recreate {
		if recreate(&mut runtime) { runtime.recreate = false; }
		else { return; }
	}

	let (image_num, acquire_future) = match acquire_next_image(runtime.other.swapchain.clone(), None) {
		Err(AcquireError::OutOfDate) => {
			runtime.recreate = true;
			return;
		},
		Err(err) => panic!("{:?}", err),
		Ok(r) => r
	};

	
	let future = create_future(&mut runtime, image_num, acquire_future);

	match future {
		Ok(future) => {
			runtime.other.previous_frame_end = Some(Box::new(future) as Box<_>);
		}
		Err(FlushError::OutOfDate) => {
			runtime.recreate = true;
			runtime.other.previous_frame_end = Some(Box::new(now(runtime.renderer.device.clone())) as Box<_>);
		}
		Err(e) => {
			println!("{:?}", e);
			runtime.other.previous_frame_end = Some(Box::new(now(runtime.renderer.device.clone())) as Box<_>);
		}
	}

	::std::thread::sleep(::std::time::Duration::from_millis(100));
}