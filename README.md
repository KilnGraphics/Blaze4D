![blaze](https://user-images.githubusercontent.com/68126718/125143247-71be4580-e0f0-11eb-88bc-070eb2838435.png)

## Information 
Blaze4D is a Fabric mod that changes Minecraft's rendering engine to use the Vulkan Graphics Library, it is currently in
Early Development and is **NOT** intended for use by the faint-hearted. Support for Blaze4D can be found in the #support
Discord channel.

We are currently in the middle of a rewrite using rust. This new version is under heavy development and as such many 
parts are still incomplete including many aspects of the build process. This can make working on the new version more 
challenging until we fix these parts.

## Community
We have a [Discord server](https://discord.gg/H93wJePuWf) where you can track development progress, ask questions, or just hang out in.

## Building
### Build Requirements

 - Vulkan SDK - Version 1.3.208 or newer
 - CMake and a C++ compiler - Required for building some of our dependencies
 - Rust - Version 1.62.0 or newer
 - Java 18

### Build Steps
The gradle rust plugin were using currently has a bug that prevents us from using it. Because of this the natives and it's
dependencies must currently be built manually.

#### Assets
To build the assets run `./gradlew :core:assets:build`. This only needs to be repeated if the assets are modified.

#### Natives
Building the natives requires to manually run cargo in a release configuration. From the project root run:
```
cd core/natives
cargo build -r
```

#### Mod
After building the natives the remainder of the project can be build and run in a single command using the standard
fabric gradle tasks.
To build the mod run `./gradlew build` back in the project root directory.
To run the game with the mod use any of the 3 run targets:
- `./gradlew runClient`
- `./gradlew runClientWithValidation` - Enables validation layers
- `./gradlew runClientWithValidationRenderdoc` - Enables validation layers and automatically loads the renderdoc shared library.

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