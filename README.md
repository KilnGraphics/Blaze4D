![blaze](https://user-images.githubusercontent.com/68126718/125143247-71be4580-e0f0-11eb-88bc-070eb2838435.png)

## Information 
Blaze4D is a Fabric mod that changes Minecraft's rendering engine to use the Vulkan Graphics Library, it is currently in
Early Development and is **NOT** intended for use by the faint-hearted. Support for Blaze4D can be found in the #support
Discord channel.

## Community
We have a [Discord server](https://discord.gg/H93wJePuWf) where you can track development progress, ask questions, or just hang out in.

## Building
### Build Requirements

 - Vulkan SDK - Version 1.3.208 or newer
 - CMake and a C++ compiler - Required for building some of our dependencies
 - Rust - Version 1.62.0 or newer
 - Java 18

### Build Steps
To build the project with natives for your platform run
```
./gradlew build
```
in the project root directory.

To run the game with the mod use any of the 3 run targets:
- `./gradlew runClient`
- `./gradlew runClientWithValidation` - Enables validation layers
- `./gradlew runClientWithValidationRenderdoc` - Enables validation layers and automatically loads the renderdoc shared library.

#### Manually building natives
To work on and test natives it can be useful to run cargo manually. To do this it's necessary to first build the assets
by running
```
./gradlew :core:assets:build
```
This only needs to be repeated if the assets are modified.

After that the natives can be manually built using cargo.

## Contributing
1. Clone the repository (https://github.com/Blaze4D-MC/Blaze4D.git).
2. Edit
3. Pull Request

## Project Structure
The project is organized in 2 parts

### Core
This is the core of Blaze4D that performs the actual rendering and is written in Rust. The gradle project contains 3
subprojects.
 - assets - These are any assets we need to bundle with Blaze4D. For example shaders or fonts. They currently need to be separately built after a change using their gradle `build` task.
 - natives - The main Blaze4D core rust code.
 - api - A java api of the rust code used by the mod.

### Mod
This is the fabric mod itself. Its job is to interface with minecraft. Most of the heavy lifting should take place in Blaze4D core.