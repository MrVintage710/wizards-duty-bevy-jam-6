use bevy::{core_pipeline::prepass::{DepthPrepass, NormalPrepass, ViewPrepassTextures}, ecs::{query::QueryItem, world::World}, prelude::*, render::{camera::CameraProjection, extract_component::ExtractComponent, render_graph::{NodeRunError, RenderGraphContext, RenderLabel}, render_resource::{binding_types, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState, ColorWrites, DynamicUniformBuffer, Extent3d, FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, UniformBuffer}, renderer::{RenderContext, RenderDevice, RenderQueue}, view::ViewTarget, Render, RenderSet}, text::cosmic_text::rustybuzz::UnicodeBuffer};

use crate::render::{RenderPhase, RenderPhaseCache};

//===============================================================================================
//          Pixelation Effect
//===============================================================================================

const PIXEL_SHADER_PATH : &str = "shaders/pixelate.wgsl";
const PASSTHROUGH_SHADER_PATH : &str = "shaders/passthrough.wgsl";

#[derive(Component, ExtractComponent, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
#[require(DepthPrepass, NormalPrepass, Msaa::Off)]
pub struct PixelationEffect {
    columns : u32,
    threshold : f32
}

impl Default for PixelationEffect {
    fn default() -> Self {
        Self {
            columns: 360,
            threshold : 0.3
        }
    }
}

impl RenderPhase for PixelationEffect {
    type Label = PixelationEffectLabel;

    type Cache = PixelationEffectCache;

    type ViewQuery = (
        &'static ViewTarget,
        &'static PixelationEffect,
        &'static ViewPrepassTextures,
        &'static Projection
    );

    fn create_bind_group_layout(_ : &World, render_device : &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(
            Some("pixelation_effect_render_pass"), 
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // The Screen texture
                    binding_types::texture_2d(TextureSampleType::Float { filterable: true }),
                    // The Normals texture
                    binding_types::texture_2d(TextureSampleType::Float { filterable: true }),
                    // The Depth texture
                    binding_types::texture_depth_2d(),
                    // The sampler that will be used to sample the screen texture
                    binding_types::sampler(SamplerBindingType::Filtering),
                    // The uniform
                    binding_types::uniform_buffer::<PixelationEffectUniform>(false),
                    // The Perspective Matrix
                    binding_types::uniform_buffer::<Mat4>(false)
                ),
            )
        )
    }

    fn fragment_shader_state(world : &World) -> Option<FragmentState> {
        let shader = world.load_asset::<Shader>(PIXEL_SHADER_PATH);

        Some(FragmentState { 
            shader, 
            shader_defs: Vec::new(), 
            entry_point: "fragment".into(), 
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        })
    }

    fn build_render_world(render_world : &mut SubApp) {
        render_world.add_systems(Render, prepare_pixel_cams.in_set(RenderSet::Prepare));
    }
    
    fn create_pipeline_descriptors(world : &World, render_device : &RenderDevice) -> Vec<RenderPipelineDescriptor> {
        vec![
            RenderPipelineDescriptor { 
                label: Some("pixilate_pipeline_descriptor".into()),
                layout: vec![Self::create_bind_group_layout(world, render_device)],
                vertex: Self::vertex_shader_state(world), 
                fragment: Self::fragment_shader_state(world),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            },
            RenderPipelineDescriptor {
                label: Some("passthrough_pipeline_descriptor".into()),
                layout: vec![Self::create_bind_group_layout(world, render_device)],
                vertex: PixelationEffect::vertex_shader_state(world), 
                fragment: Some(FragmentState {
                    shader: world.load_asset(PASSTHROUGH_SHADER_PATH),
                    shader_defs: vec![],
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            },
        ]
    }

    fn render<'w>(
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        query: QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
        cache : &'w Self::Cache,
        render_pipelines : Vec<&'w RenderPipeline>,
        bind_group_layout : &'w BindGroupLayout
    ) -> Result<(), NodeRunError> {
        
        let render_pipeline = render_pipelines[0];
        let passthrough_pipeline = render_pipelines[1];
        let (view_target, _, prepass_textures, projection) = query;
        
        //Textures toward Bind Group
        let Some(depth_texture) = prepass_textures.depth.as_ref() else { return Ok(()) };
        let Some(normal_texture) = prepass_textures.normal.as_ref() else { return Ok(()) };
        let Some((low_res_texture, _)) = cache.low_rez_texture.as_ref() else {return Ok(()) };
        let low_res_texture = low_res_texture.create_view(&TextureViewDescriptor::default());
        
        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();
        
        //Start a post process write
        let post_process_write = view_target.post_process_write();
        
        let mut projection = UniformBuffer::from(projection.get_clip_from_view().inverse());
        projection.write_buffer(device, queue);
        
        // First render pass, this renders the scene to the small texutre. This will also add all scene effects
        {
            let pixel_bind_group = render_context.render_device().create_bind_group(
                "pixelate_bind_group", 
                bind_group_layout, 
                &BindGroupEntries::sequential((
                    post_process_write.source,
                    &normal_texture.texture.default_view,
                    &depth_texture.texture.default_view, 
                    &cache.sampler,
                    cache.get_uniforms(),
                    &projection
                ))
            );
            
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor { 
                label: Some("pixelate_render_pass"), 
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &low_res_texture,
                    resolve_target: None,
                    ops: Operations::default(),
                })], 
                depth_stencil_attachment: None, 
                timestamp_writes: None, 
                occlusion_query_set: None 
            });
            
            render_pass.set_render_pipeline(render_pipeline);
            render_pass.set_bind_group(0, &pixel_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        
        //Second pass, this renders the small texture to screen. This should use just a pass through shader.
        {
            let pixel_bind_group = render_context.render_device().create_bind_group(
                "pixelate_bind_group", 
                bind_group_layout, 
                &BindGroupEntries::sequential((
                    &low_res_texture,
                    &normal_texture.texture.default_view,
                    &depth_texture.texture.default_view, 
                    &cache.sampler,
                    cache.get_uniforms(),
                    &projection
                ))
            );
            
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("pixelate_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &post_process_write.destination,
                    resolve_target: None,
                    ops: Operations::default(),
                })], 
                depth_stencil_attachment: None, 
                timestamp_writes: None,
                occlusion_query_set: None 
            });
            
            render_pass.set_render_pipeline(passthrough_pipeline);
            render_pass.set_bind_group(0, &pixel_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        
        Ok(())
    }
    
}

#[derive(Default, Debug, RenderLabel, Hash, PartialEq, Eq, Clone, Copy)]
pub struct PixelationEffectLabel;

//==============================================================================================
//        PixelationEffectUniform
//==============================================================================================

#[derive(ShaderType, Default)]
pub struct PixelationEffectUniform {
    texel : Vec2,
    threshold : f32
}

//===============================================================================================
//        PixelationEffectResource
//===============================================================================================

pub struct PixelationEffectCache {
    sampler : Sampler,
    low_rez_texture : Option<(Texture, Vec2)>,
    uniform_buffer : UniformBuffer<PixelationEffectUniform>
}

impl PixelationEffectCache {
    pub fn get_uniforms(&self) -> &UniformBuffer<PixelationEffectUniform> {
        &self.uniform_buffer
    }
}

impl FromWorld for PixelationEffectCache {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        Self {
            sampler,
            low_rez_texture : None,
            uniform_buffer: UniformBuffer::<PixelationEffectUniform>::from_world(world)
        }
    }
}

//===============================================================================================
//          Render World Systems
//===============================================================================================

fn prepare_pixel_cams(
    mut cache : ResMut<RenderPhaseCache<PixelationEffect>>,
    view_target : Single<(&ViewTarget, &PixelationEffect)>,
    render_device : Res<RenderDevice>,
    render_queue : Res<RenderQueue>,
    mut last_columns : Local<u32>
) {
    fn create_texture(render_device : &RenderDevice, width : u32, height : u32, columns : u32) -> (Texture, Vec2) {
        let columns = columns.max(1);
        let pixels_per_pixel = (width as f32 / columns as f32).round();
        
        let target_width = (width as f32 / pixels_per_pixel).ceil() as u32;
        let target_height = (height as f32 / pixels_per_pixel).ceil() as u32;
        
        let low_rez_texture = render_device.create_texture(&TextureDescriptor {
            label: Some("low_rez_texture"),
            size: Extent3d {
                width : target_width,
                height : target_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::bevy_default()],
        });
        return (low_rez_texture, Vec2::new(target_width as f32, target_height as f32))
    }
    
    let width = view_target.0.main_texture().width();
    let height = view_target.0.main_texture().height();
    
    //If the columns have changed
    
    if let Some((texture, extent)) = cache.data.low_rez_texture.as_mut() {
        if *last_columns != view_target.1.columns || width != extent.x as u32 || height != extent.y as u32 {
            let (new_texture, target_res) = create_texture(render_device.as_ref(), width, height, view_target.1.columns);
            texture.destroy();
            *texture = new_texture;
            *extent = target_res;
            *last_columns = view_target.1.columns;
            cache.data.uniform_buffer.set(PixelationEffectUniform {
                texel: Vec2::new(1.0 / target_res.x, 1.0 / target_res.y),
                threshold : view_target.1.threshold
            });
            cache.data.uniform_buffer.write_buffer(render_device.as_ref(), render_queue.as_ref());
        }
    } else {
        let (new_texture, target_res) = create_texture(render_device.as_ref(), width, height, view_target.1.columns);
        cache.data.uniform_buffer.set(PixelationEffectUniform {
            texel: Vec2::new(1.0 / target_res.x, 1.0 / target_res.y),
            threshold : view_target.1.threshold
        });
        cache.data.uniform_buffer.write_buffer(render_device.as_ref(), render_queue.as_ref());
        cache.data.low_rez_texture = Some((new_texture, target_res));
        *last_columns = view_target.1.columns;
        return;
    }
    
}