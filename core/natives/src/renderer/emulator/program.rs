use std::collections::HashMap;
use std::sync::Arc;
use glsl::parser::Parse;
use glsl::syntax::{PreprocessorVersion, PreprocessorVersionProfile};
use rspirv::spirv;
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
        let version = PreprocessorVersion::parse(src).map_err(|e| ShaderCreateError::FailedToParseGlslVersion{ info: e.info })?;
        if let Some(profile) = version.profile {
            if profile != PreprocessorVersionProfile::Core {
                return Err(ShaderCreateError::UnsupportedGlslProfile(profile.clone()));
            }
        }

        let mut options = CompileOptions::new().ok_or(ShaderCreateError::ShaderCInternalError)?;
        // We target opengl so that we dont need to perform any glsl transformations and then transform the spir-v
        options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
        options.set_target_spirv(SpirvVersion::V1_3);
        if version.version < 330 {
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
        let mut loader = rspirv::dr::Loader::new();
        rspirv::binary::Parser::new(bytemuck::cast_slice(src), &mut loader).parse().ok().ok_or(ShaderCreateError::SprivParseError)?;
        let mut module = loader.module();

        let mut names = HashMap::new();
        for instr in &module.debug_names {
            match instr.class.opcode {
                spirv::Op::Name => {
                    let target = instr.result_id.unwrap();
                    let name = instr.operands[0].unwrap_literal_string();
                    names.insert(target, String::from(name));
                },
                _ => {}
            }
        }
        let names = names;

        struct InOutVar<'a> {
            name: &'a str,
            ogl_location: Option<u32>,
        }

        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        for instr in &module.types_global_values {
            match instr.class.opcode {
                spirv::Op::Variable => {
                    let id = instr.result_id.unwrap();
                    let storage = instr.operands[0].unwrap_storage_class();
                    match storage {
                        spirv::StorageClass::Input => { inputs.insert(id, InOutVar {
                            name: names.get(&id).ok_or(ShaderCreateError::SpirvVariableMissingName(id))?,
                            ogl_location: None,
                        });},
                        spirv::StorageClass::Output => { outputs.insert(id, InOutVar {
                            name: names.get(&id).ok_or(ShaderCreateError::SpirvVariableMissingName(id))?,
                            ogl_location: None,
                        });},
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        module.annotations.retain(|instr| {
            match instr.class.opcode {
                spirv::Op::Decorate => {
                    let target = instr.result_id.unwrap();
                    let inout = inputs.get_mut(&target).or_else(|| outputs.get_mut(&target));
                    if let Some(var) = inout {
                        match instr.operands[0].unwrap_decoration() {
                            spirv::Decoration::Location => {
                                var.ogl_location = Some(instr.operands[1].unwrap_literal_int32());
                                false
                            },
                            _ => true
                        }
                    } else {
                        true
                    }
                },
                _ => true
            }
        });

        let inputs = inputs.into_iter().map(|(id, var)| -> Result<InputVariable, ShaderCreateError> {
            Ok(InputVariable {
                ogl_location: var.ogl_location.ok_or(ShaderCreateError::SpirvVariableMissingLocation(id))?,
                name: var.name.to_string(),
                spriv_id: id
            })
        }).collect::<Result<Vec<_>, _>>()?;

        let outputs = outputs.into_iter().map(|(id, var)| -> Result<OutputVariable, ShaderCreateError> {
            Ok(OutputVariable {
                ogl_location: var.ogl_location.ok_or(ShaderCreateError::SpirvVariableMissingLocation(id))?,
                name: var.name.to_string(),
                spriv_id: id
            })
        }).collect::<Result<Vec<_>, _>>()?;

        todo!()
    }
}

pub enum ShaderCreateError {
    ShaderCInternalError,
    FailedToParseGlslVersion{ info: String },
    UnsupportedGlslProfile(PreprocessorVersionProfile),
    ShaderCCompileError(shaderc::Error),
    SprivParseError,
    SpirvVariableMissingName(u32),
    SpirvVariableMissingLocation(u32),
}

impl From<shaderc::Error> for ShaderCreateError {
    fn from(err: shaderc::Error) -> Self {
        Self::ShaderCCompileError(err)
    }
}

pub struct Shader {

}

struct InputVariable {
    ogl_location: u32,
    name: String,
    spriv_id: u32,
}

struct OutputVariable {
    ogl_location: u32,
    name: String,
    spriv_id: u32,
}

pub struct Program {

}