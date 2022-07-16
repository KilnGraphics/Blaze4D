package graphics.kiln.blaze4d.build.assets

import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.plugins.ExtensionAware
import org.gradle.api.tasks.Delete
import java.io.File

class AssetsPlugin : Plugin<Project> {

    override fun apply(project: Project) {
        val extension = project.extensions.create("assets", AssetsPluginExtension::class.java);

        val outputDir = File(project.buildDir, "out");

        project.tasks.register("cleanBuildDir", Delete::class.java, {
            it.delete(outputDir)
        });

        project.tasks.create("compileShaders", {
            it.dependsOn("cleanBuildDir")

            it.actions.add({
                extension.getShaderCompiler().compile(project, outputDir);
            })
        });

        project.tasks.create("build", {
            it.dependsOn("compileShaders")
            it.outputs.dir(outputDir)
        })
    }
}