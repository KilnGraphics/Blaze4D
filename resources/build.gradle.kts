enum class ShaderType(val cliString: String) {
    VERTEX("vertex"),
    FRAGMENT("fragment"),
    TESSELATION_CONTROL("tesscontrol"),
    TESSELATION_EVALUATION("tesseval"),
    GEOMETRY("geometry"),
    COMPUTE("compute"),
}

class ShaderModule(source: File, type: ShaderType?) {
    var source: File = source;
    var type: ShaderType? = type;
}

abstract class CompileShaders : DefaultTask() {
    private var sources: ArrayList<ShaderModule> = ArrayList();

    fun shader(src: Any) {
        this.shader(src, null)
    }

    fun shader(src: Any, type: ShaderType?) {
        var file = project.file(src, PathValidation.FILE);

        this.sources.add(ShaderModule(file, type));
    }

    @TaskAction
    fun run() {
        this.sources.forEach {
            project.exec({
                executable("glslc")

                it.type?.let { type -> 
                    args("-fshader-stage=${type.cliString}")
                }

                args(it.source)
            });
        }
    }
}

tasks.register<CompileShaders>("compileShaders") {
    shader("debug/apply.vert", ShaderType.VERTEX)
}