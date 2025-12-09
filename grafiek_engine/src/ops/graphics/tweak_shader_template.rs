use std::any::Any;

use parameter_schema_derive::{ConfigSchema, EnumSchema};
use tweak_shader::{RenderContext, input_type::InputType};

use crate::error::Result;
use crate::registry::{FloatRange, IntEnum, IntRange, SignatureRegistery};
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs, OutputsExt};
use crate::{ExecutionContext, TextureMeta};

#[derive(EnumSchema, Default, Clone)]
pub enum TextureFormat {
    #[default]
    RGBA8,
    RGBA16,
    RGBAf32,
}

#[derive(ConfigSchema)]
pub struct ShaderConfig {
    pub format: TextureFormat,
    #[meta(IntRange { min: 1, max: 8192, step: 1 })]
    #[default(512)]
    #[noninteractive]
    pub width: i32,
    #[meta(IntRange { min: 1, max: 8192, step: 1 })]
    #[default(512)]
    #[noninteractive]
    pub height: i32,
    #[on_node_body]
    #[default(true)]
    pub preview: bool,
    #[meta(crate::registry::StringMeta { kind: crate::registry::StringKind::Glsl, multi_line: true })]
    pub source: String,
}

fn register_input(name: &str, input: &InputType, registry: &mut SignatureRegistery) {
    let name = name.to_string();
    match input {
        InputType::Float(b) => {
            registry
                .add_input::<f32>(name)
                .meta(FloatRange {
                    min: b.min,
                    max: b.max,
                    step: (b.max - b.min) / 100.0,
                })
                .default(b.default)
                .build();
        }
        InputType::Int(b, Some(labels)) => {
            registry
                .add_input::<i32>(name)
                .meta(IntEnum {
                    options: labels.clone(),
                })
                .default(b.default)
                .build();
        }
        InputType::Int(b, None) => {
            registry
                .add_input::<i32>(name)
                .meta(IntRange {
                    min: b.min,
                    max: b.max,
                    step: 1,
                })
                .default(b.default)
                .build();
        }
        InputType::Bool(d) => {
            registry
                .add_input::<bool>(name)
                .default(d.default.is_true())
                .build();
        }
        InputType::Image(_) => {
            registry.add_input::<crate::TextureHandle>(name).build();
        }
        InputType::Point(_) | InputType::Color(_) | InputType::RawBytes(_) => {
            log::warn!("Unsupported input type! we will get around to it!")
        }
    }
}

fn register_all_inputs(ctx: &RenderContext, registry: &mut SignatureRegistery) {
    for (name, input) in ctx.iter_inputs() {
        register_input(name, input, registry);
    }
}

pub trait ShaderTemplate: Any + Default + 'static {
    const SRC: &'static str;
    const OPERATOR: &'static str;
    const LABEL: &'static str;

    fn context(&self) -> Option<&RenderContext>;
    fn context_mut(&mut self) -> Option<&mut RenderContext>;
    fn set_context(&mut self, ctx: RenderContext);
}

impl<T: ShaderTemplate> OperationFactory for T {
    const LIBRARY: &'static str = "shader";
    const OPERATOR: &'static str = T::OPERATOR;
    const LABEL: &'static str = T::LABEL;

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(T::default()))
    }
}

impl TextureFormat {
    fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::RGBA8 => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::RGBA16 => wgpu::TextureFormat::Rgba16Unorm,
            TextureFormat::RGBAf32 => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

impl<T: ShaderTemplate> Operation for T {
    fn is_stateful(&self) -> bool {
        self.context().map(|c| c.is_stateful()).unwrap_or(false)
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.register_config::<ShaderConfig>();
        // Set the default source from the trait constant (index 4 = source field)
        if let Some(slot) = registry.config_mut(4) {
            slot.default_override = Some(crate::Value::String(T::SRC.to_string()));
        }

        let render_ctx = match RenderContext::new(
            T::SRC,
            wgpu::TextureFormat::Rgba8Unorm,
            &ctx.device,
            &ctx.queue,
        ) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to compile shader: {e}");
                return;
            }
        };

        register_all_inputs(&render_ctx, registry);
        self.set_context(render_ctx);

        registry
            .add_output::<crate::TextureHandle>("output")
            .meta(TextureMeta { preview: true })
            .dimensions(512, 512)
            .build();
    }

    fn configure(
        &mut self,
        ctx: &ExecutionContext,
        config: crate::value::Config,
        registry: &mut SignatureRegistery,
    ) -> Result<()> {
        let cfg = ShaderConfig::try_extract(config)?;
        let format = cfg.format.to_wgpu();
        let width = cfg.width as u32;
        let height = cfg.height as u32;

        let render_ctx = RenderContext::new(&cfg.source, format, &ctx.device, &ctx.queue)
            .map_err(|e| crate::error::Error::Script(format!("Shader compile error: {e}")))?;

        registry.clear_inputs();
        register_all_inputs(&render_ctx, registry);
        self.set_context(render_ctx);

        registry.clear_outputs();
        registry
            .add_output::<crate::TextureHandle>("output")
            .dimensions(width, height)
            .meta(TextureMeta {
                preview: cfg.preview,
            })
            .build();

        Ok(())
    }

    fn execute(
        &mut self,
        ctx: &mut ExecutionContext,
        inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<()> {
        let Some(render_ctx) = self.context_mut() else {
            return Ok(());
        };

        // Copy input values to shader uniforms
        let input_names: Vec<_> = render_ctx.iter_inputs().map(|(n, _)| n.clone()).collect();

        for (i, input) in inputs.iter().enumerate() {
            let Some(name) = input_names.get(i) else {
                continue;
            };
            let Some(mut uniform) = render_ctx.get_input_mut(name) else {
                continue;
            };
            match input {
                crate::ValueRef::F32(v) => {
                    if let Some(f) = uniform.as_float() {
                        f.current = **v;
                    }
                }
                crate::ValueRef::I32(v) => {
                    if let Some(i) = uniform.as_int() {
                        i.value.current = **v;
                    }
                }
                crate::ValueRef::Bool(v) => {
                    if let Some(b) = uniform.as_bool() {
                        b.current = if **v {
                            tweak_shader::input_type::ShaderBool::True
                        } else {
                            tweak_shader::input_type::ShaderBool::False
                        };
                    }
                }
                crate::ValueRef::Texture(handle) => {
                    if let Some(texture) = ctx.texture(handle) {
                        render_ctx.load_shared_texture(texture, name);
                    }
                }
                _ => {
                    log::error!(
                        "Unsupported type or something, I swear we are going to deal with this."
                    );
                }
            }
        }

        // Get output texture and render
        let output_handle: &mut crate::TextureHandle = outputs.extract(0)?;
        let Some(texture) = ctx.texture(output_handle) else {
            return Ok(());
        };

        let view = texture.create_view(&Default::default());
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        render_ctx.render(
            &ctx.queue,
            &ctx.device,
            &mut encoder,
            view,
            output_handle.width(),
            output_handle.height(),
        );
        ctx.queue.submit(Some(encoder.finish()));

        Ok(())
    }
}
