package me.hydos.rosella.render.resource

class CascadingResourceLoader(private val loaders: Collection<ResourceLoader>) : ResourceLoader {

    override fun loadResource(id: Identifier): Resource? {
        for (loader in loaders) {
            val resource = loader.loadResource(id)

            if (resource != null) {
                return object : Resource by resource {
                    override val loader: ResourceLoader
                        get() = this@CascadingResourceLoader
                }
            }
        }

        return null
    }
}
