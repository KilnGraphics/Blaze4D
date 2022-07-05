## Information 
Blaze4D is a Fabric mod that changes Minecraft's rendering engine to use the Vulkan Graphics Library, it is currently in
Early Development and is **NOT** intended for use by the faint-hearted. Support for Blaze4D can be found in the #support
Discord channel.

We are currently in the middle of a rewrite using rust. The rust library is located in the 
[Blaze4D-core](https://github.com/KilnGraphics/Blaze4D-core) repository. This new version is under heavy development
and as such many parts are still incomplete including many aspects of the build process. This can make working on the
new version more challenging until we fix these parts.

## Community
We have a [Discord server](https://discord.gg/H93wJePuWf) where you can track development progress, ask questions, or just hang out in.

## Building
The [natives](https://github.com/KilnGraphics/Blaze4D-core) have to be manually built first and the path to the resulting 
shared library has to be provided via the `b4d_native_path` gradle property. The remainder of the project can be built
without additional configuration using the gradle project.

Step-by-step build instructions:
1. Clone and build the [Blaze4D-core](https://github.com/KilnGraphics/Blaze4D-core) library.

2. Clone the [Blaze4D repository](https://github.com/KilnGraphics/Blaze4D) on the `new_b4d` branch.

3. Add the full path to the Blaze4D-core native library as a gradle property using whatever method you prefer.

4. Run ``gradlew build`` in the project folder.

## Contributing
1. Clone the repository (https://github.com/Blaze4D-MC/Blaze4D.git).
2. Edit
3. Pull Request
