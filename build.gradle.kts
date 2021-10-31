import org.gradle.internal.os.OperatingSystem

plugins {
	id("fabric-loom") version "0.10-SNAPSHOT"
	id("io.github.juuxel.loom-quiltflower") version "1.3.0"
	`maven-publish`
}

group = "graphics.kiln"
version = "1.0.0-SNAPSHOT"

val lwjglVersion = "3.3.0-SNAPSHOT"
val lwjglNatives = when (OperatingSystem.current()) {
	OperatingSystem.LINUX -> System.getProperty("os.arch").let {
		if (it.startsWith("arm") || it.startsWith("aarch64"))
			"natives-linux-${if (it.contains("64") || it.startsWith("armv8")) "arm64" else "arm32"}"
		else
			"natives-linux"
	}
	OperatingSystem.MAC_OS -> "natives-macos"
	OperatingSystem.WINDOWS -> System.getProperty("os.arch").let {
		if (it.contains("64"))
			"natives-windows${if (it.startsWith("aarch64")) "-arm64" else ""}"
		else
			"natives-windows-x86"
	}
	else -> throw Error("Unrecognized or unsupported Operating system. Please set \"lwjglNatives\" manually")
}

// If we're building Rosella in-tree, look that project up here
// The idea here is that you can clone a copy of Rosella into the project directly and settings.gradle.kts will
// find it and load it as a subproject. With this we can make it easy to work on both at once, being able to
// modify and debug across both projects without having to manually publish to maven local.
val rosellaProject = subprojects.firstOrNull { it.path == ":rosella" }

repositories {
	mavenCentral()

    maven {
        name = "hydos"
        url = uri("https://maven.hydos.cf/snapshots/")
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
		parchment("org.parchmentmc.data:parchment-1.17.1:2021.10.25-nightly-SNAPSHOT@zip")
	})
	modImplementation("net.fabricmc", "fabric-loader", properties["loader_version"].toString())
	modImplementation("net.fabricmc", "fabric-language-kotlin", "1.6.4+kotlin.1.5.30")

	// If we're building Rosella as part of the project for debugging, use that
	// Otherwise, fetch it from Maven
	if (rosellaProject != null) {
		implementation(rosellaProject)
	} else {
		include(implementation("graphics.kiln", "rosella", "1.2.0-SNAPSHOT"))
	}

	include(implementation("com.oroarmor", "aftermath", "1.0.0-beta"))

	include(implementation("org.joml", "joml", "1.10.1"))
	include(implementation("org.lwjgl", "lwjgl", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-shaderc", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-vma", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-vulkan", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-xxhash", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-shaderc", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-vma", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-xxhash", lwjglVersion, classifier = lwjglNatives))

	// Upgrade Minecraft's LWJGL
	include(implementation("org.lwjgl", "lwjgl", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-glfw", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-jemalloc", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-openal", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-opengl", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-stb", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl-tinyfd", lwjglVersion))
	include(implementation("org.lwjgl", "lwjgl", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-glfw", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-jemalloc", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-openal", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-opengl", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-stb", lwjglVersion, classifier = lwjglNatives))
	include(implementation("org.lwjgl", "lwjgl-tinyfd", lwjglVersion, classifier = lwjglNatives))

	if (lwjglNatives == "natives-macos" || lwjglNatives == "natives-macos-arm64") {
		include("org.lwjgl", "lwjgl-vulkan", lwjglVersion, classifier = lwjglNatives)
	}

	testImplementation("org.junit.jupiter", "junit-jupiter", "5.7.0")
}

configurations.all {
    resolutionStrategy {
        dependencySubstitution {
            substitute(module("org.lwjgl:lwjgl:3.2.2")).with(module("org.lwjgl:lwjgl:$lwjglVersion"))
            substitute(module("org.lwjgl:lwjg-glfw:3.2.2")).with(module("org.lwjgl:lwjgl-glfw:$lwjglVersion"))
        }

        force("org.lwjgl:lwjgl:$lwjglVersion")
        force("org.lwjgl:lwjgl-glfw:$lwjglVersion")
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
	test {
		useJUnitPlatform()
	}

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
			from(components["java"])

			pom {
				name.set("Rosella")
				packaging = "jar"

				description.set("A Minecraft mod to use Vulkan through the Rosella engine")
				url.set("https://github.com/KilnGraphics/Blaze4D")

				licenses {
					license {
						name.set("GNU Lesser General Public License v3.0")
						url.set("https://www.gnu.org/licenses/lgpl-3.0.txt")
					}
				}

				developers {
					developer {
						id.set("hYdos")
						name.set("Hayden V")
						email.set("haydenv06@gmail.com")
						url.set("https://hydos.cf/")
					}

					developer {
						id.set("OroArmor")
						name.set("Eli Orona")
						email.set("eliorona@live.com")
						url.set("https://oroarmor.com/")
					}

					developer {
						id.set("CodingRays")
						url.set("https://github.com/CodingRays")
					}

					developer {
						id.set("burgerdude")
						name.set("Ryan G")
						url.set("https://github.com/burgerguy")
					}

					developer {
						id.set("ramidzkh")
						email.set("ramidzkh@gmail.com")
						url.set("https://github.com/ramidzkh")
					}
				}
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

		maven {
			val releasesRepoUrl = uri("https://maven.hydos.cf/releases")
			val snapshotsRepoUrl = uri("https://maven.hydos.cf/snapshots")
			name = "hydos"
			url = if (version.toString().endsWith("SNAPSHOT")) snapshotsRepoUrl else releasesRepoUrl

			val u = System.getenv("MAVEN_USERNAME") ?: return@maven
			val p = System.getenv("MAVEN_PASSWORD") ?: return@maven

			credentials {
				username = u
				password = p
			}
		}
	}
}
