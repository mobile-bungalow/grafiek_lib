use super::tweak_shader_template::ShaderTemplate;
use tweak_shader::RenderContext;

macro_rules! shader_op {
    ($name:ident, $operator:literal, $label:literal, $path:literal) => {
        #[derive(Default)]
        pub struct $name {
            ctx: Option<RenderContext>,
        }

        impl ShaderTemplate for $name {
            const SRC: &'static str = include_str!($path);
            const OPERATOR: &'static str = $operator;
            const LABEL: &'static str = $label;

            fn context(&self) -> Option<&RenderContext> {
                self.ctx.as_ref()
            }

            fn context_mut(&mut self) -> Option<&mut RenderContext> {
                self.ctx.as_mut()
            }

            fn set_context(&mut self, ctx: RenderContext) {
                self.ctx = Some(ctx);
            }
        }
    };
}

shader_op!(Grayscale, "grayscale", "Grayscale", "glsl/grayscale.glsl");
