package me.hydos.rosella.render.model

import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import org.lwjgl.assimp.AIFile
import org.lwjgl.assimp.AIFileIO
import org.lwjgl.assimp.AIScene
import org.lwjgl.assimp.Assimp.*
import org.lwjgl.system.MemoryUtil

fun loadScene(resource: Resource, flags: Int): AIScene? {
	val identifier = resource.identifier

	val context = identifier.path.run { substring(0, lastIndexOf('/')) }
	val name = identifier.path.run { substring(lastIndexOf('/') + 1) }

	val io = AIFileIO.create().apply {
		OpenProc { _, nFileName, _ ->
			val fileName = MemoryUtil.memASCII(nFileName)
			val id = Identifier(identifier.namespace, context + fileName)
			val data = resource.loader.ensureResource(id).readAllBytes(true)

			AIFile.create().apply {
				ReadProc { _, pBuffer, size, count ->
					val max = (data.remaining().toLong() / size).coerceAtMost(count)
					MemoryUtil.memCopy(MemoryUtil.memAddress(data), pBuffer, max * size)
					data.position(data.position() + (max * size).toInt())
					max
				}

				SeekProc { _, offset, origin ->
					when (origin) {
						aiOrigin_CUR -> {
							data.position(data.position() + offset.toInt())
						}
						aiOrigin_SET -> {
							data.position(offset.toInt())
						}
						aiOrigin_END -> {
							data.position(data.limit() + offset.toInt())
						}
					}

					0
				}

				TellProc { data.position().toLong() }
				FileSizeProc { data.limit().toLong() }

				FlushProc {
					error("Cannot flush")
				}

				WriteProc { _, _, _, _ ->
					error("Cannot write")
				}
			}.address()
		}
		CloseProc { _, pFile ->
			val file = AIFile.create(pFile)

			file.FlushProc().free()
			file.SeekProc().free()
			file.FileSizeProc().free()
			file.TellProc().free()
			file.WriteProc().free()
			file.ReadProc().free()
		}
	}

	val scene = aiImportFileEx("/$name", flags, io)

	io.OpenProc().free()
	io.CloseProc().free()

	return scene
}
