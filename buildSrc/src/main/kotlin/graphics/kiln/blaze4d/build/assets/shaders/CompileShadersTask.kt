package graphics.kiln.blaze4d.build.assets.shaders

import graphics.kiln.blaze4d.build.assets.AssetsPlugin
import graphics.kiln.blaze4d.build.assets.AssetsPluginExtension
import org.gradle.api.DefaultTask
import org.gradle.api.tasks.TaskAction
import java.io.File
import javax.inject.Inject

abstract class CompileShadersTask : DefaultTask() {
    init {
        outputs.dir(project.buildDir)
    }

    @TaskAction
    fun compile() {
        var assetsPlugin = project.plugins.getPlugin(AssetsPlugin::class.java);

        project.delete(assetsPlugin.outputDir);
        assetsPlugin.extension.getShaderCompiler().compile(project, assetsPlugin.outputDir);
    }
}