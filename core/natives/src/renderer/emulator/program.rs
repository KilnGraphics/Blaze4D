use std::sync::Arc;
use glsl::parser::Parse;
use glsl::syntax::{PreprocessorVersion, PreprocessorVersionProfile};
use shaderc::{CompileOptions, Compiler, EnvVersion, GlslProfile, ShaderKind, SpirvVersion, TargetEnv};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    pub fn as_shader_kind(self) -> ShaderKind {
        match self {
            ShaderStage::Vertex => ShaderKind::Vertex,
            ShaderStage::Fragment => ShaderKind::Fragment,
        }
    }
}

pub struct ProgramManager {

}

impl ProgramManager {
    pub fn create_shader_from_glsl(src: &str, stage: ShaderStage, name: &str) -> Result<Arc<Shader>, ShaderCreateError> {
        let version = PreprocessorVersion::parse(src).map_err(|e| ShaderCreateError::FailedToParseVersion{ info: e.info })?;
        if let Some(profile) = version.profile {
            if profile != PreprocessorVersionProfile::Core {
                return Err(ShaderCreateError::UnsupportedGlslProfile(profile.clone()));
            }
        }

        let mut options = CompileOptions::new().ok_or(ShaderCreateError::ShaderCInternalError)?;
        // We target opengl so that we dont need to perform any glsl transformations and then transform the spir-v
        options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
        options.set_target_spirv(SpirvVersion::V1_3);
        if version < 330 {
            options.set_forced_version_profile(330, GlslProfile::Core);
        }
        options.set_auto_bind_uniforms(true);
        options.set_auto_map_locations(true);
        options.set_auto_combined_image_sampler(true);

        let compiler = Compiler::new().ok_or(ShaderCreateError::ShaderCInternalError)?;
        let compiled = compiler.compile_into_spirv(src, stage.as_shader_kind(), name, "main", None)?;
        Self::create_shader_from_spirv(compiled.as_binary())
    }

    pub fn create_shader_from_spirv(src: &[u32]) -> Result<Arc<Shader>, ShaderCreateError> {
        todo!()
    }
}

pub enum ShaderCreateError {
    ShaderCInternalError,
    FailedToParseGlslVersion{ info: String },
    UnsupportedGlslProfile(PreprocessorVersionProfile),
    ShaderCCompileError(shaderc::Error),
}

impl From<shaderc::Error> for ShaderCreateError {
    fn from(err: shaderc::Error) -> Self {
        Self::ShaderCCompileError(err)
    }
}

pub struct Shader {

}

pub struct Program {

}