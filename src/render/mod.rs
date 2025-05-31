use std::{marker::PhantomData, result::Result};

use bevy::{core_pipeline::{core_3d::graph::{Core3d, Node3d}, fullscreen_vertex_shader::fullscreen_shader_vertex_state}, ecs::query::{QueryItem, ReadOnlyQueryData}, prelude::*, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, render_graph::{IntoRenderNodeArray, RenderGraphApp, RenderGraphContext, RenderLabel, RenderSubGraph, ViewNode, ViewNodeRunner}, render_resource::{BindGroupLayout, CachedRenderPipelineId, FragmentState, MultisampleState, PipelineCache, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderType, VertexState}, renderer::{RenderContext, RenderDevice}, RenderApp}};

pub mod pixelate;

pub const VERTEX_SHADER_ENTRY_POINT : &str = "vertex";
pub const FRAGMENT_SHADER_ENTRY_POINT : &str = "fragment";

//===============================================================================================
//          Post Process Effect Trait
//===============================================================================================

pub trait RenderPhase : ExtractComponent + Clone {

    type Label : RenderLabel + Default;

    type Cache : FromWorld + Send + Sync;

    type ViewQuery : ReadOnlyQueryData;

    fn get_subgraph() -> impl RenderSubGraph {
        Core3d
    }

    fn node_ordering() -> impl IntoRenderNodeArray<3> {
        (Node3d::Tonemapping, Self::Label::default(), Node3d::EndMainPassPostProcessing)
    }
    
    fn add_subgraph(render_world : &mut SubApp) {
        render_world
            .add_render_graph_node::<ViewNodeRunner<RenderPhaseNode<Self>>>(Self::get_subgraph(), Self::Label::default())
            .add_render_graph_edges(Self::get_subgraph(), Self::node_ordering());
        ;
    }

    fn plugin() -> impl Plugin {
        RenderPhasePlugin::<Self>::default()
    }

    fn vertex_shader_state(_world : &World) -> VertexState {
        fullscreen_shader_vertex_state()
    }

    fn fragment_shader_state(_world : &World) -> Option<FragmentState> {
        None
    }
    
    fn create_pipeline_descriptors(world : &World, render_device : &RenderDevice) -> Vec<RenderPipelineDescriptor> {
        vec![
            RenderPipelineDescriptor { 
                label: Some(std::any::type_name::<Self>().into()), 
                layout: vec![Self::create_bind_group_layout(world, render_device)],
                vertex: Self::vertex_shader_state(world), 
                fragment: Self::fragment_shader_state(world),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            }
        ]
    }

    fn build(_app : &mut App) {}

    fn build_render_world(_render_world : &mut SubApp) {}

    fn create_bind_group_layout(world : &World, render_device : &RenderDevice) -> BindGroupLayout;

    fn render<'w>(
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        query: QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
        cache : &'w Self::Cache,
        render_pipelines : Vec<&'w RenderPipeline>,
        bind_group_layout : &'w BindGroupLayout
    ) -> Result<(), bevy::render::render_graph::NodeRunError>;
}

//===============================================================================================
//          Into Uniform
//===============================================================================================

pub trait IntoUniform {
    type UniformType : ShaderType;

    fn into_uniform(self) -> Self::UniformType;
}

impl <S : ShaderType> IntoUniform for S {
    type UniformType = S;

    fn into_uniform(self) -> Self::UniformType {
        self
    }
}

//===============================================================================================
//          Post Process Effect Plugin
//===============================================================================================

pub struct RenderPhasePlugin<P : RenderPhase>(PhantomData<P>);

impl <P : RenderPhase> Default for RenderPhasePlugin<P> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl <P> Plugin for RenderPhasePlugin<P> where P : RenderPhase {
    fn build(&self, app: &mut App) {

        app.add_plugins(ExtractComponentPlugin::<P>::default());

        P::build(app);
        
        let Some(render_world) = app.get_sub_app_mut(RenderApp) else { return };
        P::add_subgraph(render_world);

        P::build_render_world(render_world);
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<RenderPhaseCache<P>>();

    }
}

//===============================================================================================
//          Post Process Effect Plugin
//===============================================================================================

pub struct RenderPhaseNode<P : RenderPhase>(PhantomData<P>);

impl <P : RenderPhase> Default for RenderPhaseNode<P> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl <P : RenderPhase> ViewNode for RenderPhaseNode<P> {
    type ViewQuery = P::ViewQuery;

    fn run<'w>(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        view_query: bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let Some(resource) = world.get_resource::<RenderPhaseCache<P>>() else {return Ok(())};
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = resource.pipelines.iter().filter_map(|pipeline| pipeline_cache.get_render_pipeline(*pipeline)).collect::<Vec<_>>();

        P::render(graph, render_context, view_query, world, &resource.data, pipelines, &resource.bind_group)
    }
}

//===============================================================================================
//          Post Process Resource
//===============================================================================================

#[derive(Resource)]
pub struct RenderPhaseCache<P : RenderPhase> {
    pub pipelines: Vec<CachedRenderPipelineId>,
    pub bind_group: BindGroupLayout,
    pub data : P::Cache
}

impl <P : RenderPhase> FromWorld for RenderPhaseCache<P> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>().clone();

        let bind_group = P::create_bind_group_layout(world, &render_device);
        
        let pipelines = {
            let descriptors = P::create_pipeline_descriptors(world, &render_device);
            descriptors.into_iter().map(|descriptor| world.resource_mut::<PipelineCache>().queue_render_pipeline(descriptor)).collect()
        };
    
        let data = P::Cache::from_world(world);

        RenderPhaseCache { pipelines, bind_group, data }
    }
}
