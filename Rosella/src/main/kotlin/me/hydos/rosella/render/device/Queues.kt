package me.hydos.rosella.render.device

import org.lwjgl.vulkan.VkQueue

class Queues {
	lateinit var graphicsQueue: VkQueue
	lateinit var presentQueue: VkQueue
}