package graphics.kiln.blaze4d.build.assets.shaders

import org.gradle.api.Action
import org.gradle.api.NamedDomainObjectContainer
import org.gradle.api.Project
import org.gradle.api.file.RelativePath
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import java.io.File

abstract class ShaderCompiler {

    @get:Input
    public abstract val targetSpirv: Property<SprivVersion>;

    @get:Input
    public abstract val sourceDir: Property<RelativePath>;

    @get:Input
    public abstract val outputDir: Property<RelativePath>;

    @get:Input
    public abstract val projects: NamedDomainObjectContainer<ShaderProject>;

    public fun targetSpriv(version: SprivVersion) {
        this.targetSpirv.set(version)
    }

    public fun sourceDir(path: String) {
        this.sourceDir.set(RelativePath.parse(false, path))
    }

    public fun outputDir(path: String) {
        this.outputDir.set(RelativePath.parse(false, path))
    }

    public fun addProject(name: String, configure: Action<ShaderProject>) {
        this.projects.create(name, configure)
    }

    fun compile(project: Project, outBaseDir: File) {
        var srcBaseDir = this.sourceDir.getOrElse(RelativePath.parse(false, "src")).getFile(project.projectDir);
        var newOutBaseDir = this.outputDir.getOrElse(RelativePath.parse(false, "")).getFile(outBaseDir);

        var targetSpirv = this.targetSpirv.getOrElse(SprivVersion.SPV_1_0);
        var config = CompilerConfig(targetSpirv, HashSet());

        this.projects.forEach({
            it.compile(project, srcBaseDir, newOutBaseDir, config)
        })
    }
}