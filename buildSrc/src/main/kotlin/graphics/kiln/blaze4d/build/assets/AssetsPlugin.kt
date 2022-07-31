package graphics.kiln.blaze4d.build.assets

import org.gradle.api.Action
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.plugins.ExtensionAware
import org.gradle.api.tasks.Delete

import java.io.File

import graphics.kiln.blaze4d.build.assets.shaders.CompileShadersTask;

class AssetsPlugin : Plugin<Project> {

    public lateinit var extension: AssetsPluginExtension;
    public lateinit var outputDir: File;

    override fun apply(project: Project) {
        extension = project.extensions.create("assets", AssetsPluginExtension::class.java);
        outputDir = File(project.buildDir, "out");

        project.tasks.register("compileShaders", CompileShadersTask::class.java);

        project.tasks.create("build", {
            it.dependsOn("compileShaders")
        });

        project.afterEvaluate({
            project.tasks.getByName("compileShaders", {
                it.inputs.dir(extension.getShaderCompiler().generateSourceDir(project));
            })
        })
    }
}