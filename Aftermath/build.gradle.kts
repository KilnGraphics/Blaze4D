import org.gradle.internal.os.OperatingSystem

plugins {
    `java-library`
}

group "me.hydos"
version "1.1.0"

val lwjglNatives = when (OperatingSystem.current()) {
    OperatingSystem.LINUX -> System.getProperty("os.arch").let {
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

val lwjglVersion = "3.3.0-SNAPSHOT"

dependencies {
    implementation(platform("org.lwjgl:lwjgl-bom:$lwjglVersion"))
    implementation("org.lwjgl", "lwjgl")
    implementation("org.jetbrains:annotations:20.1.0")
    runtimeOnly("org.lwjgl", "lwjgl", classifier = lwjglNatives)
}