package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.scene.`object`.RenderObject
import org.joml.Matrix4f
import org.joml.Vector3f
import org.lwjgl.system.MemoryUtil

open class GuiRenderObject(
    material: Material,
    private var z: Float = -1f,
    private var colour: Vector3f = Vector3f(0f, 0f, 0f),
    viewMatrix: Matrix4f,
    projectionMatrix: Matrix4f
) : RenderObject(Resource.Empty as Resource?, material, projectionMatrix, viewMatrix) {

    constructor(
        material: Material,
        z: Float,
        colour: Vector3f,
        scaleX: Float,
        scaleZ: Float,
        viewMatrix: Matrix4f,
        projectionMatrix: Matrix4f
    ) : this(material, z, colour, viewMatrix, projectionMatrix) {
        scale(scaleX, scaleZ)
    }

    constructor(
        material: Material,
        z: Float,
        colour: Vector3f,
        scaleX: Float,
        scaleZ: Float,
        translateX: Float,
        translateZ: Float,
        viewMatrix: Matrix4f,
        projectionMatrix: Matrix4f
    ) : this(material, z, colour, scaleX, scaleZ, viewMatrix, projectionMatrix) {
        translate(translateX, translateZ)
    }

    override fun loadModelInfo() {
        val size = material.pipeline.vertexFormat.size
        vertexBuffer = MemoryUtil.memAlloc(size * 4)
        colour = Vector3f(0f, 0f, 0f)

        vertexBuffer
            .putFloat(-0.5f).putFloat(-0.5f).putFloat(0f)
            .putFloat(colour.x).putFloat(colour.y).putFloat(colour.z)
            .putFloat(0f).putFloat(0f)

        vertexBuffer
            .putFloat(0.5f).putFloat(-0.5f).putFloat(0f)
            .putFloat(colour.x).putFloat(colour.y).putFloat(colour.z)
            .putFloat(1f).putFloat(0f)

        vertexBuffer
            .putFloat(0.5f).putFloat(0.5f).putFloat(0f)
            .putFloat(colour.x).putFloat(colour.y).putFloat(colour.z)
            .putFloat(1f).putFloat(1f)

        vertexBuffer
            .putFloat(-0.5f).putFloat(0.5f).putFloat(0f)
            .putFloat(colour.x).putFloat(colour.y).putFloat(colour.z)
            .putFloat(0f).putFloat(1f)
        this.vertexBuffer.rewind()

        this.indices = MemoryUtil.memAlloc(6 * Integer.BYTES)
        this.indices.putInt(0)
        this.indices.putInt(1)
        this.indices.putInt(2)
        this.indices.putInt(2)
        this.indices.putInt(3)
        this.indices.putInt(0)
        this.indices.rewind()
    }

    override fun onAddedToScene(rosella: Rosella) {
        super.onAddedToScene(rosella)
        modelMatrix.translate(0f, 0f, z)
    }

    private fun scale(x: Float, y: Float) {
        modelMatrix.scale(x, y, 1f)
    }

    private fun translate(x: Float, y: Float) {
        modelMatrix.translate(x, -y, 0f)
    }
}
