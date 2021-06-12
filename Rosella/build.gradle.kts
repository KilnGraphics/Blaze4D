import org.gradle.internal.os.OperatingSystem

plugins {
	java
	kotlin("jvm") version "1.5.10"
	id("com.github.johnrengelman.shadow") version "7.0.0"
}

group = "me.hydos"
version = "1.0-SNAPSHOT"
val lwjglVersion = "3.3.0-SNAPSHOT"

val lwjglNatives = when (OperatingSystem.current()) {
	OperatingSystem.LINUX   -> System.getProperty("os.arch").let {
		if (it.startsWith("arm") || it.startsWith("aarch64"))
			"natives-linux-${if (it.contains("64") || it.startsWith("armv8")) "arm64" else "arm32"}"
		else
			"natives-linux"
	}
	OperatingSystem.MAC_OS  -> if (System.getProperty("os.arch").startsWith("aarch64")) "natives-macos-arm64" else "natives-macos"
	OperatingSystem.WINDOWS -> "natives-windows"
	else -> throw Error("Unrecognized or unsupported Operating system. Please set \"lwjglNatives\" manually")
}

repositories {
	mavenCentral()
	maven("https://oss.sonatype.org/content/repositories/snapshots/")
}

dependencies {
	implementation(platform("org.lwjgl:lwjgl-bom:$lwjglVersion"))

	implementation("org.lwjgl", "lwjgl")
	implementation("org.lwjgl", "lwjgl-assimp")
	implementation("org.lwjgl", "lwjgl-glfw")
	implementation("org.lwjgl", "lwjgl-openal")
	implementation("org.lwjgl", "lwjgl-shaderc")
	implementation("org.lwjgl", "lwjgl-stb")
	implementation("org.lwjgl", "lwjgl-vma")
	implementation("org.lwjgl", "lwjgl-vulkan")

	implementation("org.joml", "joml", "1.10.1")
	implementation("it.unimi.dsi", "fastutil", "8.5.4")
	implementation("com.google.code.gson", "gson", "2.8.7")

	runtimeOnly("org.lwjgl", "lwjgl", classifier = lwjglNatives)
	runtimeOnly("org.lwjgl", "lwjgl-assimp", classifier = lwjglNatives)
	runtimeOnly("org.lwjgl", "lwjgl-glfw", classifier = lwjglNatives)
	runtimeOnly("org.lwjgl", "lwjgl-openal", classifier = lwjglNatives)
	runtimeOnly("org.lwjgl", "lwjgl-shaderc", classifier = lwjglNatives)
	runtimeOnly("org.lwjgl", "lwjgl-stb", classifier = lwjglNatives)
	runtimeOnly("org.lwjgl", "lwjgl-vma", classifier = lwjglNatives)
	if (lwjglNatives == "natives-macos" || lwjglNatives == "natives-macos-arm64") runtimeOnly("org.lwjgl", "lwjgl-vulkan", classifier = lwjglNatives)
}