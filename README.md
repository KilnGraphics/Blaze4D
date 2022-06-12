![blaze](https://user-images.githubusercontent.com/68126718/125143247-71be4580-e0f0-11eb-88bc-070eb2838435.png)

## Information
Blaze4D is a Fabric mod that changes Minecraft's rendering engine to use the Vulkan Graphics Library, it is currently in
Early Development and is NOT intended for use by the faint-hearted.

This repository is the rust core which performs all render work.

## Community
We have a [Discord server](https://discord.gg/H93wJePuWf) where you can track development progress, ask questions, or just hang out in.

## Building
### Additional Dependencies
 - Vulkan SDK
 - A c++ compiler and CMake (This is required to build the vulkan profiles library)
 - Gradle

### Build instructions
1. Compile all resources (shaders, fonts etc.) by running `./gradlew` in the `resources` directory.
2. Compile Blaze4D-core by running `cargo build -r`.