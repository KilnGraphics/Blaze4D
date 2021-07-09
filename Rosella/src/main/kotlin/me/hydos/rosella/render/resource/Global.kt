package me.hydos.rosella.render.resource

import me.hydos.rosella.Rosella
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.nio.ByteBuffer
import javax.imageio.ImageIO

/**
 * Don't use this once [Rosella] get its own ResourceLoader field
 */
object Global : ResourceLoader by ClassLoaderResourceLoader(ClassLoader.getSystemClassLoader()) {

    fun fromBufferedImage(image: BufferedImage, id: Identifier): Resource {
        return object : Resource {
            override val identifier: Identifier
                get() = id

            override val loader: ResourceLoader
                get() = this@Global

            override fun openStream(): InputStream {
                val out = ByteArrayOutputStream()
                ImageIO.write(
                    image,
                    "png",
                    out
                )
                return ByteArrayInputStream(out.toByteArray())
            }
        }
    }

    fun fromByteBuffer(bb: ByteBuffer, id: Identifier): Resource {
        return object : Resource {
            override val identifier: Identifier
                get() = id

            override val loader: ResourceLoader
                get() = this@Global

            override fun openStream(): InputStream {
                val byteArray = ByteArray(bb.remaining())
                bb.get(byteArray)
                return ByteArrayInputStream(byteArray)
            }
        }
    }
}
