package me.hydos.rosella.render.resource

import java.io.InputStream

class ClassLoaderResourceLoader(private val loader: ClassLoader) : ResourceLoader {

    override fun loadResource(id: Identifier): Resource? {
        val url = loader.getResource(id.file)

        return if (url == null) {
            null
        } else object : Resource {
            override val identifier: Identifier
                get() = id

            override val loader: ResourceLoader
                get() = this@ClassLoaderResourceLoader

            override fun openStream(): InputStream = url.openStream()
        }
    }
}
