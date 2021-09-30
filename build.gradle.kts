import org.gradle.internal.os.OperatingSystem

plugins {
	id("fabric-loom") version "0.9-SNAPSHOT"
	id("io.github.juuxel.loom-quiltflower") version "1.3.0"
	`maven-publish`
}

group = "me.hydos"
version = "1.0.0-SNAPSHOT"

val lwjglVersion = "3.3.0-SNAPSHOT"
val lwjglNatives = getCurrentNatives(OperatingSystem.current())

fun getCurrentNatives(system: OperatingSystem): String {
	when {
		system.isLinux -> {
			System.getProperty("os.arch").let {
				return when {
					it.startsWith("arm") || it.startsWith("aarch64") -> {
						val arch = when {
							it.contains("64") || it.startsWith("armv8") -> {
								"arm64"
							}
							else -> {
								"arm32"
							}
						}

						"natives-linux-$arch"
					}
					else -> {
						"natives-linux"
					}
				}
			}
		}
		system.isMacOsX -> {
			return when {
				System.getProperty("os.arch").startsWith("aarch64") -> "natives-macos-arm64"
				else -> "natives-macos"
			}
		}
		system.isWindows -> {
			return "natives-windows"
		}
	}

	error("Unrecognized or unsupported Operating system. Please set \"lwjglNatives\" manually")
}

allprojects {
	extra["lwjgl.version"] = lwjglVersion
	extra["lwjgl.natives"] = lwjglNatives
}

repositories {
	mavenCentral()

    maven {
        name = "hydos's maven"
        url = uri("http://150.242.33.216/snapshots")
        isAllowInsecureProtocol = true
    }

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
	mappings(loom.layered {
		officialMojangMappings()
		parchment("org.parchmentmc.data:parchment-1.17.1:2021.07.26-nightly-SNAPSHOT@zip")
	})
	modImplementation("net.fabricmc", "fabric-loader", properties["loader_version"].toString())
	modImplementation("net.fabricmc", "fabric-language-kotlin", "1.6.4+kotlin.1.5.30")

	include(implementation("kiln.graphics", "rosella", "1.1.0"))
	include(implementation("com.oroarmor", "aftermath", "1.0.0-beta"))

	include("org.joml", "joml", "1.10.1")
	include("org.lwjgl", "lwjgl-shaderc", lwjglVersion)
	include("org.lwjgl", "lwjgl-vma", lwjglVersion)
	include("org.lwjgl", "lwjgl-vulkan", lwjglVersion)
	include("org.lwjgl", "lwjgl-xxhash", lwjglVersion)
	include("org.lwjgl", "lwjgl-shaderc", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-vma", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-xxhash", lwjglVersion, classifier = lwjglNatives)

	// Upgrade Minecraft's LWJGL
	include("org.lwjgl", "lwjgl", lwjglVersion)
	include("org.lwjgl", "lwjgl-glfw", lwjglVersion)
	include("org.lwjgl", "lwjgl-jemalloc", lwjglVersion)
	include("org.lwjgl", "lwjgl-openal", lwjglVersion)
	include("org.lwjgl", "lwjgl-opengl", lwjglVersion)
	include("org.lwjgl", "lwjgl-stb", lwjglVersion)
	include("org.lwjgl", "lwjgl-tinyfd", lwjglVersion)

	include("org.lwjgl", "lwjgl", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-glfw", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-jemalloc", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-openal", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-opengl", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-stb", lwjglVersion, classifier = lwjglNatives)
	include("org.lwjgl", "lwjgl-tinyfd", lwjglVersion, classifier = lwjglNatives)

	if (lwjglNatives == "natives-macos" || lwjglNatives == "natives-macos-arm64") {
		include("org.lwjgl", "lwjgl-vulkan", lwjglVersion, classifier = lwjglNatives)
	}

	testImplementation("org.junit.jupiter", "junit-jupiter", "5.7.0")
}

tasks.test {
	useJUnitPlatform {
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
	accessWidenerPath.set(file("src/main/resources/blaze4d.aw"))

	runs {
		val client by this

		create("clientWithValidation") {
			inherit(client)
			configName = "Minecraft Client with Validation Layers"
			vmArgs.add("-Drosella:validation=true")
		}

		create("clientWithRenderdoc") {
			inherit(client)
			configName = "Minecraft Client with Renderdoc"
			vmArgs.add("-Drosella:renderdoc=true")
		}

		create("clientWithValidationRenderdoc") {
			inherit(client)
			configName = "Minecraft Client with Validation Layers and Renderdoc"
			vmArgs.add("-Drosella:validation=true")
			vmArgs.add("-Drosella:renderdoc=true")
		}
	}
}

quiltflower {
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
