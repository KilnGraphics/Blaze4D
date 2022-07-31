package graphics.kiln.blaze4d.build.assets.shaders

import org.gradle.api.file.RelativePath
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import java.io.File

abstract class ShaderModule {
    public abstract fun getName(): String;

    @get:Input
    public abstract val srcFile: Property<RelativePath>;

    @get:Input
    public abstract val outFile: Property<RelativePath>;

    public fun srcFile(file: String) {
        this.srcFile.set(RelativePath.parse(true, file))
    }

    fun generateArg(srcBaseDir: File, outBaseDir: File): Array<String> {
        var relativeSrcFile = this.srcFile.getOrElse(RelativePath.parse(true, this.getName()));
        var srcFile = relativeSrcFile.getFile(srcBaseDir);

        var generatedOutFile = "${srcFile.nameWithoutExtension}_${srcFile.extension}.spv";
        var outFile = this.outFile.getOrElse(relativeSrcFile.replaceLastName(generatedOutFile)).getFile(outBaseDir);

        var outDir = outFile.parentFile;
        if(!outDir.exists()) {
            if (!outDir.mkdirs()) {
                throw java.lang.RuntimeException("Failed to create output directory for shader module. ${outDir}");
            }
        }

        return arrayOf("${srcFile.absolutePath}", "-o${outFile.absolutePath}");
    }
}