package graphics.kiln.blaze4d.build.assets.shaders

enum class ShaderStage(cliArg: String) {
    VERTEX("-fshader-stage=vertex"),
    FRAGMENT("-fshader-stage=fragment"),
    TESSELATION_CONTROL("-fshader-stage=tesscontrol"),
    TESSELATION_EVALUATION("-fshader-stage=tesseval"),
    GEOMETRY("-fshader-stage=geometry"),
    COMPUTE("-fshader-stage=compute"),
}