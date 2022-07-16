package graphics.kiln.blaze4d.build.assets.shaders

import java.io.File

data class CompilerConfig(
        val targetSpriv: SprivVersion,
        val includeDirs: Set<File>,
)
