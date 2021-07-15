plugins {
    java
    kotlin("jvm") version "1.5.10"
    id("com.github.johnrengelman.shadow") version "7.0.0"
}

group = "me.hydos"
version = "1.0-SNAPSHOT"

val lwjglVersion = extra["lwjgl.version"].toString()
val lwjglNatives = extra["lwjgl.natives"].toString()

repositories {
    mavenCentral()

    maven {
        name = "Sonatype Snapshots"
        url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }
}

dependencies {
    api(platform("org.lwjgl:lwjgl-bom:$lwjglVersion"))

    api("org.lwjgl", "lwjgl")
    api("org.lwjgl", "lwjgl-assimp")
    api("org.lwjgl", "lwjgl-glfw")
    api("org.lwjgl", "lwjgl-openal")
    api("org.lwjgl", "lwjgl-shaderc")
    api("org.lwjgl", "lwjgl-stb")
    api("org.lwjgl", "lwjgl-vma")
    api("org.lwjgl", "lwjgl-vulkan")

    api("org.joml", "joml", "1.10.1")
    api("it.unimi.dsi", "fastutil", "8.5.4")
    api("com.google.code.gson", "gson", "2.8.7")
    api("org.apache.logging.log4j", "log4j-core", "2.14.1")

    runtimeOnly("org.lwjgl", "lwjgl", classifier = lwjglNatives)
    runtimeOnly("org.lwjgl", "lwjgl-assimp", classifier = lwjglNatives)
    runtimeOnly("org.lwjgl", "lwjgl-glfw", classifier = lwjglNatives)
    runtimeOnly("org.lwjgl", "lwjgl-openal", classifier = lwjglNatives)
    runtimeOnly("org.lwjgl", "lwjgl-shaderc", classifier = lwjglNatives)
    runtimeOnly("org.lwjgl", "lwjgl-stb", classifier = lwjglNatives)
    runtimeOnly("org.lwjgl", "lwjgl-vma", classifier = lwjglNatives)

    if (lwjglNatives == "natives-macos" || lwjglNatives == "natives-macos-arm64") {
        runtimeOnly("org.lwjgl", "lwjgl-vulkan", classifier = lwjglNatives)
    }

    testImplementation("org.junit.jupiter:junit-jupiter:5.7.1")
}

tasks.test {
    useJUnitPlatform {
    }
}

tasks.register<Test>("fastCITest") {
    useJUnitPlatform {
        excludeTags("exclude_frequent_ci")
    }
}

tasks.register<Test>("slowCITest") {
    useJUnitPlatform {
    } // In the future we can add tags to exclude tests that require certain vulkan features which arent available on github

}