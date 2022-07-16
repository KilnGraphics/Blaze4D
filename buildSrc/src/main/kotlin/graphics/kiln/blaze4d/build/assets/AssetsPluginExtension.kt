package graphics.kiln.blaze4d.build.assets

import graphics.kiln.blaze4d.build.assets.shaders.ShaderCompiler
import org.gradle.api.Action
import org.gradle.api.tasks.Nested

abstract class AssetsPluginExtension {

    @Nested
    public abstract fun getShaderCompiler(): ShaderCompiler;

    fun shaders(configure: Action<ShaderCompiler>) {
        configure.execute(this.getShaderCompiler());
    }
}