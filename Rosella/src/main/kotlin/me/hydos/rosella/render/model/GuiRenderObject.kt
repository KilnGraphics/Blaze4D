package me.hydos.rosella.render.model

import it.unimi.dsi.fastutil.ints.IntArrayList
import me.hydos.rosella.Rosella
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.vertex.BufferVertexConsumer
import me.hydos.rosella.scene.`object`.RenderObject
import org.joml.Matrix4f
import org.joml.Vector3f

open class GuiRenderObject(
    material: Material,
    private var z: Float = -1f,
    private var colour: Vector3f = Vector3f(0f, 0f, 0f),
    viewMatrix: Matrix4f,
    projectionMatrix: Matrix4f
) : RenderObject(Resource.Empty, material, viewMatrix, projectionMatrix) {

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
        // FIXME FIXME FIXME FIXME this is bad, don't force cast here
        val vertexConsumer: BufferVertexConsumer = renderInfo.bufferProvider as BufferVertexConsumer

        vertexConsumer.clear()
        renderInfo.indices = IntArrayList()

        colour = Vector3f(0f, 0f, 0f)

        // TODO: is this conversion doing what it should be? should convert int representing unsigned byte to signed byte through wrapping
        vertexConsumer
            .pos(-0.5f, -0.5f, 0f)
            .color(colour.x().toInt().toByte(), colour.y().toInt().toByte(), colour.z().toInt().toByte())
            .uv(0f, 0f)
            .nextVertex()

        vertexConsumer
            .pos(0.5f, -0.5f, 0f)
            .color(colour.x().toInt().toByte(), colour.y().toInt().toByte(), colour.z().toInt().toByte())
            .uv(1f, 0f)
            .nextVertex()

        vertexConsumer
            .pos(0.5f, 0.5f, 0f)
            .color(colour.x().toInt().toByte(), colour.y().toInt().toByte(), colour.z().toInt().toByte())
            .uv(1f, 1f)
            .nextVertex()

        vertexConsumer
            .pos(-0.5f, 0.5f, 0f)
            .color(colour.x().toInt().toByte(), colour.y().toInt().toByte(), colour.z().toInt().toByte())
            .uv(0f, 1f)
            .nextVertex()

        renderInfo.indices.add(0)
        renderInfo.indices.add(1)
        renderInfo.indices.add(2)
        renderInfo.indices.add(2)
        renderInfo.indices.add(3)
        renderInfo.indices.add(0)
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
