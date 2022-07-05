package graphics.kiln.blaze4d.core.types;

import graphics.kiln.blaze4d.core.natives.VertexFormatNative;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;

import java.util.Optional;

public class B4DVertexFormat implements AutoCloseable {

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    public B4DVertexFormat() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(VertexFormatNative.LAYOUT, this.resourceScope);
    }

    public void initialize() {
        this.setStride(0);
        this.setPosition(0, B4DFormat.UNDEFINED);
        this.setNormal();
        this.setColor();
        this.setUV0();
        this.setUV1();
        this.setUV2();
    }

    public void setStride(int stride) {
        VertexFormatNative.STRIDE_HANDLE.set(this.memory, stride);
    }

    public void setPosition(FormatEntry entry) {
        this.setPosition(entry.offset, entry.format);
    }

    public void setPosition(int offset, B4DFormat format) {
        this.setPosition(offset, format.getValue());
    }

    public void setPosition(int offset, int format) {
        VertexFormatNative.POSITION_OFFSET_HANDLE.set(this.memory, offset);
        VertexFormatNative.POSITION_FORMAT_HANDLE.set(this.memory, format);
    }

    public FormatEntry getPosition() {
        int offset = (int) VertexFormatNative.POSITION_OFFSET_HANDLE.get(this.memory);
        int format = (int) VertexFormatNative.POSITION_FORMAT_HANDLE.get(this.memory);
        return new FormatEntry(offset, B4DFormat.fromRaw(format));
    }

    public void setNormal() {
        VertexFormatNative.HAS_NORMAL_HANDLE.set(this.memory, false);
    }

    public void setNormal(FormatEntry entry) {
        this.setNormal(entry.offset, entry.format);
    }

    public void setNormal(int offset, B4DFormat format) {
        this.setNormal(offset, format.getValue());
    }

    public void setNormal(int offset, int format) {
        VertexFormatNative.NORMAL_OFFSET_HANDLE.set(this.memory, offset);
        VertexFormatNative.NORMAL_FORMAT_HANDLE.set(this.memory, format);
        VertexFormatNative.HAS_NORMAL_HANDLE.set(this.memory, true);
    }

    public Optional<FormatEntry> getNormal() {
        if ((boolean) VertexFormatNative.HAS_NORMAL_HANDLE.get(this.memory)) {
            int offset = (int) VertexFormatNative.NORMAL_OFFSET_HANDLE.get(this.memory);
            int format = (int) VertexFormatNative.NORMAL_FORMAT_HANDLE.get(this.memory);
            return Optional.of(new FormatEntry(offset, B4DFormat.fromRaw(format)));
        }
        return Optional.empty();
    }

    public void setColor() {
        VertexFormatNative.HAS_COLOR_HANDLE.set(this.memory, false);
    }

    public void setColor(FormatEntry entry) {
        this.setColor(entry.offset, entry.format);
    }

    public void setColor(int offset, B4DFormat format) {
        this.setColor(offset, format.getValue());
    }

    public void setColor(int offset, int format) {
        VertexFormatNative.COLOR_OFFSET_HANDLE.set(this.memory, offset);
        VertexFormatNative.COLOR_FORMAT_HANDLE.set(this.memory, format);
        VertexFormatNative.HAS_COLOR_HANDLE.set(this.memory, true);
    }

    public Optional<FormatEntry> getColor() {
        if ((boolean) VertexFormatNative.HAS_COLOR_HANDLE.get(this.memory)) {
            int offset = (int) VertexFormatNative.COLOR_OFFSET_HANDLE.get(this.memory);
            int format = (int) VertexFormatNative.COLOR_FORMAT_HANDLE.get(this.memory);
            return Optional.of(new FormatEntry(offset, B4DFormat.fromRaw(format)));
        }
        return Optional.empty();
    }

    public void setUV0() {
        VertexFormatNative.HAS_UV0_HANDLE.set(this.memory, false);
    }

    public void setUV0(FormatEntry entry) {
        this.setUV0(entry.offset, entry.format);
    }

    public void setUV0(int offset, B4DFormat format) {
        this.setUV0(offset, format.getValue());
    }

    public void setUV0(int offset, int format) {
        VertexFormatNative.UV0_OFFSET_HANDLE.set(this.memory, offset);
        VertexFormatNative.UV0_FORMAT_HANDLE.set(this.memory, format);
        VertexFormatNative.HAS_UV0_HANDLE.set(this.memory, true);
    }

    public Optional<FormatEntry> getUV0() {
        if ((boolean) VertexFormatNative.HAS_UV0_HANDLE.get(this.memory)) {
            int offset = (int) VertexFormatNative.UV0_OFFSET_HANDLE.get(this.memory);
            int format = (int) VertexFormatNative.UV0_FORMAT_HANDLE.get(this.memory);
            return Optional.of(new FormatEntry(offset, B4DFormat.fromRaw(format)));
        }
        return Optional.empty();
    }

    public void setUV1() {
        VertexFormatNative.HAS_UV1_HANDLE.set(this.memory, false);
    }

    public void setUV1(FormatEntry entry) {
        this.setUV1(entry.offset, entry.format);
    }

    public void setUV1(int offset, B4DFormat format) {
        this.setUV1(offset, format.getValue());
    }

    public void setUV1(int offset, int format) {
        VertexFormatNative.UV1_OFFSET_HANDLE.set(this.memory, offset);
        VertexFormatNative.UV1_FORMAT_HANDLE.set(this.memory, format);
        VertexFormatNative.HAS_UV1_HANDLE.set(this.memory, true);
    }

    public Optional<FormatEntry> getUV1() {
        if ((boolean) VertexFormatNative.HAS_UV1_HANDLE.get(this.memory)) {
            int offset = (int) VertexFormatNative.UV1_OFFSET_HANDLE.get(this.memory);
            int format = (int) VertexFormatNative.UV1_FORMAT_HANDLE.get(this.memory);
            return Optional.of(new FormatEntry(offset, B4DFormat.fromRaw(format)));
        }
        return Optional.empty();
    }

    public void setUV2() {
        VertexFormatNative.HAS_UV2_HANDLE.set(this.memory, false);
    }

    public void setUV2(FormatEntry entry) {
        this.setUV2(entry.offset, entry.format);
    }

    public void setUV2(int offset, B4DFormat format) {
        this.setUV2(offset, format.getValue());
    }

    public void setUV2(int offset, int format) {
        VertexFormatNative.UV2_OFFSET_HANDLE.set(this.memory, offset);
        VertexFormatNative.UV2_FORMAT_HANDLE.set(this.memory, format);
        VertexFormatNative.HAS_UV2_HANDLE.set(this.memory, true);
    }

    public Optional<FormatEntry> getUV2() {
        if ((boolean) VertexFormatNative.HAS_UV2_HANDLE.get(this.memory)) {
            int offset = (int) VertexFormatNative.UV2_OFFSET_HANDLE.get(this.memory);
            int format = (int) VertexFormatNative.UV2_FORMAT_HANDLE.get(this.memory);
            return Optional.of(new FormatEntry(offset, B4DFormat.fromRaw(format)));
        }
        return Optional.empty();
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }

    public record FormatEntry(int offset, B4DFormat format) {
    }
}
