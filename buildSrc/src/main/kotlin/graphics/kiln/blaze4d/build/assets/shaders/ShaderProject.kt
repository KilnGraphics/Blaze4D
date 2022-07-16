package graphics.kiln.blaze4d.build.assets.shaders

import org.gradle.api.Action
import org.gradle.api.NamedDomainObjectContainer
import org.gradle.api.Project
import org.gradle.api.file.RelativePath
import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import org.gradle.api.tasks.Input

import java.io.File

abstract class ShaderProject {
    public abstract fun getName(): String;

    @get:Input
    public abstract val projectDir: Property<RelativePath>;

    @get:Input
    public abstract val targetSpriv: Property<SprivVersion>;

    @get:Input
    public abstract val includeDirs: SetProperty<RelativePath>;

    @get:Input
    public abstract val modules: NamedDomainObjectContainer<ShaderModule>

    public fun projectDir(path: String) {
        this.projectDir.set(RelativePath.parse(false, path))
    }

    public fun targetSpriv(version: SprivVersion) {
        this.targetSpriv.set(version)
    }

    public fun addIncludeDir(path: String) {
        this.includeDirs.add(RelativePath.parse(false, path))
    }

    public fun addModule(name: String) {
        this.modules.create(name)
    }

    public fun addModule(name: String, configure: Action<ShaderModule>) {
        this.modules.create(name, configure)
    }

    fun compile(project: Project, srcBaseDir: File, outBaseDir: File, parentConfig: CompilerConfig) {
        val projectDir = this.projectDir.getOrElse(RelativePath.parse(false, ""));
        val newSrcBaseDir = projectDir.getFile(srcBaseDir);
        val newOutBaseDir = projectDir.getFile(outBaseDir);

        val targetSpriv = this.targetSpriv.getOrElse(parentConfig.targetSpriv);
        val includeDirs = HashSet<File>(parentConfig.includeDirs);
        this.includeDirs.orNull?.forEach({
            includeDirs.add(it.getFile(newSrcBaseDir));
        });
        includeDirs.add(newSrcBaseDir);
        val config = CompilerConfig(targetSpriv, includeDirs);

        this.execCompile(project, newSrcBaseDir, newOutBaseDir, config);
    }

    private fun execCompile(project: Project, srcBaseDir: File, outBaseDir: File, config: CompilerConfig) {
        this.modules.forEach({
            var moduleArgs = it.generateArg(srcBaseDir, outBaseDir);

            project.exec({
                it.executable("glslc")
                it.args(config.targetSpriv.cliArg)

                var exec = it;
                config.includeDirs.forEach({
                    exec.args("-I${it}")
                })

                moduleArgs.forEach({
                    exec.args(it)
                })
            })
        })
    }
}