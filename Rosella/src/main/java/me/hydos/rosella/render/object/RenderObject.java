package me.hydos.rosella.render.object;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.ModelLoader;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.ubo.RenderObjectUbo;
import me.hydos.rosella.render.util.memory.Memory;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.render.vertex.VertexFormats;
import me.hydos.rosella.vkobjects.VkCommon;
import org.joml.Matrix4f;
import org.joml.Vector2fc;
import org.joml.Vector3f;
import org.joml.Vector3fc;
import org.lwjgl.assimp.Assimp;

import java.util.ArrayList;

public class RenderObject implements Renderable {

    private final Material material;
    private final Resource modelId;
    public final RenderInfo renderInfo = new RenderInfo(new BufferVertexConsumer(VertexFormats.Companion.getPOSITION_COLOR_UV()));
    public InstanceInfo instanceInfo;

    public Matrix4f modelMatrix = new Matrix4f();
    public Matrix4f viewMatrix = new Matrix4f();
    public Matrix4f projectionMatrix = new Matrix4f();

    public RenderObject(Resource model, Material material) {
        this.material = material;
        this.modelId = model;
        loadModelInfo();
    }

    public void loadModelInfo() {
        ModelLoader.SimpleModel model = ModelLoader.loadModel(modelId, Assimp.aiProcess_FlipUVs | Assimp.aiProcess_DropNormals);
        int vertexCount = model.getPositions().size();

        renderInfo.consumer.clear();
        Vector3f color = new Vector3f(1.0f, 1.0f, 1.0f);
        for (int i = 0; i < vertexCount; i++) {
            Vector3fc pos = model.getPositions().get(i);
            Vector2fc uvs = model.getTexCoords().get(i);
            renderInfo.consumer
                    .pos(pos.x(), pos.y(), pos.z())
                    .color((int) color.x(), (int) color.y(), (int) color.z())
                    .uv(uvs.x(), uvs.y())
                    .nextVertex();
        }

        renderInfo.indices = new ArrayList<>(model.getIndices().size());
        renderInfo.indices.addAll(model.getIndices());
    }

    @Override
    public void onAddedToScene(VkCommon common, Memory memory) {
        instanceInfo = new InstanceInfo(new RenderObjectUbo(common.device, memory, this, material.shader), material);
        throw new RuntimeException("E");
//        this.projectionMatrix = rosella.getCamera().getProj();
//        this.viewMatrix = rosella.getCamera().getView();
    }

    @Override
    public void free(Memory memory, Device device) {
        instanceInfo.free(device, memory);
        renderInfo.free(device, memory);
    }

    @Override
    public void rebuild(Rosella rosella) {
        instanceInfo.rebuild(rosella);
    }

    @Override
    public InstanceInfo getInstanceInfo() {
        return instanceInfo;
    }

    @Override
    public RenderInfo getRenderInfo() {
        return renderInfo;
    }
}
