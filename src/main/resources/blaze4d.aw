accessWidener   v1  named

# Some Misc Image Stuff
accessible method net/minecraft/client/texture/NativeImage$Format getFormat (I)Lnet/minecraft/client/texture/NativeImage$Format;
accessible  method   net/minecraft/client/texture/NativeImage   <init>    (Lnet/minecraft/client/texture/NativeImage$Format;IIZJ)V

# Make Minecraft Shut Up
accessible class net/minecraft/client/resource/VideoWarningManager$WarningPatternLoader

# Access to Matrix4f so i can convert Minecraft matrices to JOML matrices
accessible  field   net/minecraft/util/math/Matrix4f   a00   F
accessible  field   net/minecraft/util/math/Matrix4f   a01   F
accessible  field   net/minecraft/util/math/Matrix4f   a02   F
accessible  field   net/minecraft/util/math/Matrix4f   a03   F

accessible  field   net/minecraft/util/math/Matrix4f   a10   F
accessible  field   net/minecraft/util/math/Matrix4f   a11   F
accessible  field   net/minecraft/util/math/Matrix4f   a12   F
accessible  field   net/minecraft/util/math/Matrix4f   a13   F

accessible  field   net/minecraft/util/math/Matrix4f   a20   F
accessible  field   net/minecraft/util/math/Matrix4f   a21   F
accessible  field   net/minecraft/util/math/Matrix4f   a22   F
accessible  field   net/minecraft/util/math/Matrix4f   a23   F

accessible  field   net/minecraft/util/math/Matrix4f   a30   F
accessible  field   net/minecraft/util/math/Matrix4f   a31   F
accessible  field   net/minecraft/util/math/Matrix4f   a32   F
accessible  field   net/minecraft/util/math/Matrix4f   a33   F

# Access To Sprites For Textures
accessible  field   net/minecraft/client/texture/SpriteAtlasTexture$Data   spriteIds Ljava/util/Set;
accessible  field   net/minecraft/client/texture/SpriteAtlasTexture$Data   width I
accessible  field   net/minecraft/client/texture/SpriteAtlasTexture$Data   height I
accessible  field   net/minecraft/client/texture/SpriteAtlasTexture$Data   maxLevel I
accessible  field   net/minecraft/client/texture/SpriteAtlasTexture$Data   sprites   Ljava/util/List;

# Access to texture stuff for texture rendering
accessible class com/mojang/blaze3d/platform/GlStateManager$Texture2DState
accessible class net/minecraft/client/gui/screen/SplashOverlay$LogoTexture
accessible  method  net/minecraft/client/texture/ResourceTexture   upload    (Lnet/minecraft/client/texture/NativeImage;ZZ)V

accessible class net/minecraft/client/texture/ResourceTexture$TextureData
accessible field net/minecraft/client/texture/TextureManager resourceContainer Lnet/minecraft/resource/ResourceManager;
