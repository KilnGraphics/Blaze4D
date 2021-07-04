package me.hydos.rosella.render.texture

import it.unimi.dsi.fastutil.ints.IntArrayPriorityQueue
import it.unimi.dsi.fastutil.ints.IntPriorityQueues
import me.hydos.rosella.Rosella
import me.hydos.rosella.render.*
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.renderer.Renderer
import org.lwjgl.vulkan.VK10

/**
 * Caches Textures and other texture related objects
 */
class TextureManager(val device: Device) { // TODO: add layers, maybe not in this class but somewhere

	private val textureMap = HashMap<Int, Texture>()
	private val samplerCache = HashMap<SamplerCreateInfo, TextureSampler>() // bro there's like 3 options for this

	private val preparedTextures = HashSet<Texture>()

	private val reusableTexIds = IntPriorityQueues.synchronize(IntArrayPriorityQueue())
	private var nextTexId : Int = 0;

	fun generateTextureId(): Int {
		return if (!reusableTexIds.isEmpty) {
			reusableTexIds.dequeueInt()
		} else {
			nextTexId++
		}
	}

	fun deleteTexture(textureId: Int) {
		val removedTex = textureMap.remove(textureId)
		preparedTextures.remove(removedTex)
		reusableTexIds.enqueue(textureId)
	}

	fun getTexture(textureId: Int): Texture? {
		return textureMap[textureId];
	}

	fun createTexture(
		engine: Rosella,
		textureId: Int,
		width: Int,
		height: Int,
		imgFormat: Int,
		samplerCreateInfo: SamplerCreateInfo
	) {
		val textureImage = TextureImage(0, 0, 0)
		createTextureImage(engine.renderer, width, height, imgFormat, textureImage)
		textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)

		val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
			TextureSampler(samplerCreateInfo, engine.device)
		}

		textureMap[textureId] = Texture(imgFormat, width, height, textureImage, textureSampler.pointer);
	}

	fun applySamplerInfoToTexture(
		device: Device,
		textureId: Int,
		samplerCreateInfo: SamplerCreateInfo
	) {
		val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
			TextureSampler(samplerCreateInfo, device)
		}

		textureMap[textureId]?.textureSampler = textureSampler.pointer
	}

	fun drawToExistingTexture(
		engine: Rosella,
		textureId: Int,
		image: UploadableImage,
		srcRegion: ImageRegion,
		dstRegion: ImageRegion,
	) {
		val texture = getTexture(textureId)!!
		if (preparedTextures.contains(texture)) {
			transitionImageLayout(
				texture.textureImage.textureImage,
				texture.imgFormat,
				VK10.VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
				VK10.VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
				engine.renderer.depthBuffer,
				engine.renderer.device,
				engine.renderer
			)
			preparedTextures.remove(texture)
		}
		drawToTexture(engine.device, image, srcRegion, dstRegion, engine.renderer, engine.memory, texture)
	}

	fun drawToExistingTexture(
		engine: Rosella,
		textureId: Int,
		image: UploadableImage
	) {
		val region = ImageRegion(0, 0, image.getWidth(), image.getHeight())
		drawToExistingTexture(engine, textureId, image, region, region)
	}

	fun prepareTexture(renderer: Renderer, texture: Texture) {
		if (!preparedTextures.contains(texture)) {
			transitionImageLayout(
				texture.textureImage.textureImage,
				texture.imgFormat,
				VK10.VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
				VK10.VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
				renderer.depthBuffer,
				renderer.device,
				renderer
			)
			preparedTextures.add(texture)
		}
	}
}