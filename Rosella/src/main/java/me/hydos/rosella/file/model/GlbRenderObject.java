package me.hydos.rosella.file.model;

import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.scene.object.RenderObject;
import org.joml.Matrix4f;
import org.joml.Vector2fc;
import org.joml.Vector3f;
import org.joml.Vector3fc;
import org.lwjgl.system.MemoryUtil;

public class GlbRenderObject extends RenderObject {

    private final GlbModelLoader.MeshData meshData;

    protected GlbRenderObject(Material material, GlbModelLoader.MeshData meshData, Matrix4f modelViewMatrix, Matrix4f projectionMatrix) {
        super(Resource.Empty.INSTANCE, material, projectionMatrix, modelViewMatrix);
        this.meshData = meshData;
        int vertexCount = meshData.positions.size();
        int size = material.pipeline().getVertexFormat().getSize();
        this.vertexBuffer = MemoryUtil.memAlloc(size * vertexCount);
        Vector3f color = new Vector3f(1.0f, 1.0f, 1.0f);

        for (int i = 0; i < vertexCount; i++) {
            Vector3fc pos = meshData.positions.get(i);
            Vector2fc uvs = meshData.texCoords.get(i);

            vertexBuffer
                    .putFloat(pos.x())
                    .putFloat(pos.y())
                    .putFloat(pos.z());

            vertexBuffer
                    .putFloat(color.x())
                    .putFloat(color.y())
                    .putFloat(color.z());

            vertexBuffer
                    .putFloat(uvs.x())
                    .putFloat(uvs.y());
        }

        this.indices = MemoryUtil.memAlloc(meshData.indices.size() * Integer.BYTES);
        for (Integer index : meshData.indices) {
            this.indices.putInt(index);
        }
        this.indices.rewind();
        this.vertexBuffer.rewind();
    }

    @Override
    public void loadModelInfo() {

    }
}
