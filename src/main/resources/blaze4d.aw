accessWidener   v1  named

# Make animated textures stop animating
accessible class net/minecraft/client/renderer/texture/TextureAtlasSprite$AnimatedTexture

# Make Minecraft Shut Up
accessible class net/minecraft/client/renderer/GpuWarnlistManager$Preparations

# Access to Matrix4f so i can convert Minecraft matrices to JOML matrices
accessible  field   com/mojang/math/Matrix4f   m00   F
accessible  field   com/mojang/math/Matrix4f   m01   F
accessible  field   com/mojang/math/Matrix4f   m02   F
accessible  field   com/mojang/math/Matrix4f   m03   F

accessible  field   com/mojang/math/Matrix4f   m10   F
accessible  field   com/mojang/math/Matrix4f   m11   F
accessible  field   com/mojang/math/Matrix4f   m12   F
accessible  field   com/mojang/math/Matrix4f   m13   F

accessible  field   com/mojang/math/Matrix4f   m20   F
accessible  field   com/mojang/math/Matrix4f   m21   F
accessible  field   com/mojang/math/Matrix4f   m22   F
accessible  field   com/mojang/math/Matrix4f   m23   F

accessible  field   com/mojang/math/Matrix4f   m30   F
accessible  field   com/mojang/math/Matrix4f   m31   F
accessible  field   com/mojang/math/Matrix4f   m32   F
accessible  field   com/mojang/math/Matrix4f   m33   F

# Access to ChunkInfo for world rendering
accessible class net/minecraft/client/renderer/LevelRenderer$RenderChunkInfo