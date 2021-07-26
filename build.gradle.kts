import org.gradle.internal.os.OperatingSystem

plugins {
    id("fabric-loom") version "0.9.25"
    id("io.github.juuxel.loom-quiltflower") version "1.1.1"
    `maven-publish`
}

group = "me.hydos"
version = "1.0.0-SNAPSHOT"

val lwjglVersion = "3.3.0-SNAPSHOT"
val lwjglNatives = when (OperatingSystem.current()) {
    OperatingSystem.LINUX -> System.getProperty("os.arch").let {
        if (it.startsWith("arm") || it.startsWith("aarch64")) {
            val arch = if (it.contains("64") || it.startsWith("armv8")) {
                "arm64"
            } else {
                "arm32"
            }

            "natives-linux-$arch"
        } else {
            "natives-linux"
        }
    }
    OperatingSystem.MAC_OS -> if (System.getProperty("os.arch")
            .startsWith("aarch64")
    ) "natives-macos-arm64" else "natives-macos"
    OperatingSystem.WINDOWS -> "natives-windows"
    else -> error("Unrecognized or unsupported Operating system. Please set \"lwjglNatives\" manually")
}

allprojects {
    extra["lwjgl.version"] = lwjglVersion
    extra["lwjgl.natives"] = lwjglNatives
}

repositories {
    mavenCentral()

    maven {
        name = "Sonatype Snapshots"
        url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }

    maven {
        name = "ldtteam"
        url = uri("https://ldtteam.jfrog.io/artifactory/parchmentmc-snapshots/")
    }
}

dependencies {
    minecraft("net.minecraft", "minecraft", properties["minecraft_version"].toString())
    mappings (loom.layered {
        officialMojangMappings()
        parchment("org.parchmentmc.data:parchment-1.17.1:2021.07.26-nightly-SNAPSHOT@zip")
    })
    modImplementation("net.fabricmc", "fabric-loader", properties["loader_version"].toString())

    include(implementation(project(":Rosella"))!!)
    include(implementation("com.oroarmor", "aftermath", "1.0.0-beta"))

    include("org.joml", "joml", "1.10.1")
    include("org.lwjgl", "lwjgl-shaderc", lwjglVersion)
    include("org.lwjgl", "lwjgl-vma", lwjglVersion)
    include("org.lwjgl", "lwjgl-vulkan", lwjglVersion)
    include("org.lwjgl", "lwjgl-shaderc", lwjglVersion, classifier = lwjglNatives)
    include("org.lwjgl", "lwjgl-vma", lwjglVersion, classifier = lwjglNatives)

    if (lwjglNatives == "natives-macos" || lwjglNatives == "natives-macos-arm64") {
        include("org.lwjgl", "lwjgl-vulkan", lwjglVersion, classifier = lwjglNatives)
    }
}

base {
    archivesBaseName = "blaze4d"
}

java {
    sourceCompatibility = JavaVersion.VERSION_16
    targetCompatibility = JavaVersion.VERSION_16

    withSourcesJar()
}

loom {
    accessWidener("src/main/resources/blaze4d.aw")
}

loomQuiltflower {
    quiltflowerVersion.set("1.5.0")
}

tasks {
    withType<JavaCompile> {
        options.encoding = "UTF-8"
        options.release.set(16)
    }

    withType<AbstractArchiveTask> {
        from(file("LICENSE"))
    }

    processResources {
        inputs.property("version", project.version)

        filesMatching("fabric.mod.json") {
            expand("version" to project.version)
        }
    }
}

publishing {
    publications {
        create<MavenPublication>("mod") {
            artifact(tasks.remapJar)
            artifact(tasks.remapSourcesJar) {
                classifier = "sources"
            }
        }
    }

    repositories {
        maven {
            val releasesRepoUrl = uri("${buildDir}/repos/releases")
            val snapshotsRepoUrl = uri("${buildDir}/repos/snapshots")
            name = "Project"
            url = if (version.toString().endsWith("SNAPSHOT")) snapshotsRepoUrl else releasesRepoUrl
        }
    }
}
