apply<graphics.kiln.blaze4d.build.assets.AssetsPlugin>()

configure<graphics.kiln.blaze4d.build.assets.AssetsPluginExtension> {
    shaders {
        targetSpriv(graphics.kiln.blaze4d.build.assets.shaders.SprivVersion.SPV_1_3)

        addProject("Emulator") {
            projectDir("emulator")

            addModule("debug/position.vert")
            addModule("debug/color.vert")
            addModule("debug/uv.vert")
            addModule("debug/null.vert")
            addModule("debug/debug.frag")
            addModule("debug/textured.frag")
            addModule("debug/background.vert")
            addModule("debug/background.frag")
        }

        addProject("Utils") {
            projectDir("utils")

            addModule("full_screen_quad.vert")
            addModule("blit.frag")
        }

        addProject("Debug") {
            projectDir("debug")

            addModule("apply.vert")
            addModule("apply.frag")
            addModule("font/msdf_font.vert")
            addModule("font/msdf_font.frag")
            addModule("basic.vert")
            addModule("basic.frag")
        }
    }
}