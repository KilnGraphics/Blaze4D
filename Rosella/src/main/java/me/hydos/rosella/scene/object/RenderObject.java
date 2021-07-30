package me.hydos.rosella.scene.object;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.ManagedBuffer;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.ModelLoader;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.ubo.RenderObjectUbo;
import org.joml.Matrix4f;
import org.joml.Vector2fc;
import org.joml.Vector3f;
import org.joml.Vector3fc;
import org.lwjgl.assimp.Assimp;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Future;

public class RenderObject implements Renderable {

    protected final Material material;
    private final Resource modelId;
    public Future<RenderInfo> renderInfo;
    public InstanceInfo instanceInfo;

    public final Matrix4f modelMatrix = new Matrix4f();
    public final Matrix4f viewMatrix;
    public final Matrix4f projectionMatrix;
    protected ByteBuffer indices;
    protected ByteBuffer vertexBuffer;

    public RenderObject(Resource model, Material material, Matrix4f projectionMatrix, Matrix4f viewMatrix) {
        this.material = material;
        this.modelId = model;
        this.projectionMatrix = projectionMatrix;
        this.viewMatrix = viewMatrix;
        loadModelInfo();
    }

    public void loadModelInfo() {
        ModelLoader.SimpleModel model = ModelLoader.loadModel(modelId, Assimp.aiProcess_FlipUVs | Assimp.aiProcess_DropNormals);
        int vertexCount = model.getPositions().size();
        int size = material.pipelineCreateInfo().vertexFormat().getSize();
        this.vertexBuffer = MemoryUtil.memAlloc(size * vertexCount);
        Vector3f color = new Vector3f(1.0f, 1.0f, 1.0f);

        for (int i = 0; i < vertexCount; i++) {
            Vector3fc pos = model.getPositions().get(i);
            Vector2fc uvs = model.getTexCoords().get(i);

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

        this.indices = MemoryUtil.memAlloc(model.getIndices().size() * Integer.BYTES);
        for (Integer index : model.getIndices()) {
            this.indices.putInt(index);
        }
        this.indices.rewind();
        this.vertexBuffer.rewind();
    }

    @Override
    public void onAddedToScene(Rosella rosella) {
        instanceInfo = new InstanceInfo(new RenderObjectUbo(rosella.common.device, rosella.common.memory, this, material.pipelineCreateInfo().shaderProgram()), material);
        renderInfo = CompletableFuture.completedFuture(new RenderInfo(
                rosella.bufferManager.createVertexBuffer(new ManagedBuffer<>(vertexBuffer, true)),
                rosella.bufferManager.createIndexBuffer(new ManagedBuffer<>(indices, true)),
                indices.capacity() / 4
        ));
    }

    @Override
    public void free(VulkanDevice device, Memory memory) {
        instanceInfo.free(device, memory);
        try {
            renderInfo.get().free(device, memory);
        } catch (InterruptedException | ExecutionException e) {
            Rosella.LOGGER.error("Error freeing render info", e);
        }
    }

    @Override
    public void rebuild(Rosella rosella) {
        instanceInfo.rebuild(rosella);
    }

    @Override
    public void hardRebuild(Rosella rosella) {
        instanceInfo.hardRebuild(rosella);
    }

    @Override
    public InstanceInfo getInstanceInfo() {
        return instanceInfo;
    }

    @Override
    public Future<RenderInfo> getRenderInfo() {
        return renderInfo;
    }
}
