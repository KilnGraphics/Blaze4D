package me.hydos.blaze4d.test;

import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.block.entity.BlockEntity;
import net.minecraft.fluid.FluidState;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import net.minecraft.world.BlockRenderView;
import net.minecraft.world.BlockView;
import net.minecraft.world.chunk.ChunkProvider;
import net.minecraft.world.chunk.light.LightingProvider;
import net.minecraft.world.level.ColorResolver;
import org.jetbrains.annotations.Nullable;

public class FakeWorld implements BlockRenderView {
    @Override
    public float getBrightness(Direction direction, boolean shaded) {
        return 10;
    }

    @Override
    public LightingProvider getLightingProvider() {
        return new FakeLightingProvider();
    }

    @Override
    public int getColor(BlockPos pos, ColorResolver colorResolver) {
        return 0xFF00FF00;
    }

    @Nullable
    @Override
    public BlockEntity getBlockEntity(BlockPos pos) {
        throw new RuntimeException("Dummy World Impl");
    }

    @Override
    public BlockState getBlockState(BlockPos pos) {
        return Blocks.GRASS_BLOCK.getDefaultState();
    }

    @Override
    public FluidState getFluidState(BlockPos pos) {
        throw new RuntimeException("Dummy World Impl");
    }

    @Override
    public int getHeight() {
        return 256;
    }

    @Override
    public int getBottomY() {
        return 0;
    }

    public static class FakeLightingProvider extends LightingProvider {

        public FakeLightingProvider() {
            super(new FakeChunkProvider(), true, true);
        }
    }

    public static class FakeChunkProvider implements ChunkProvider {

        @Nullable
        @Override
        public BlockView getChunk(int chunkX, int chunkZ) {
            return null;
        }

        @Override
        public BlockView getWorld() {
            return null;
        }
    }
}
