import org.gradle.internal.impldep.org.junit.experimental.categories.Categories.CategoryFilter.include
import org.gradle.internal.os.OperatingSystem

plugins {
	id("fabric-loom") version "0.12-SNAPSHOT"
	//id("io.github.juuxel.loom-quiltflower-mini") version "1.2.1"
	`maven-publish`
}

group = "graphics.kiln"
version = "1.0.0-SNAPSHOT"

repositories {
	mavenCentral()

	maven {
		name = "Sonatype Snapshots"
		url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
	}

    maven {
        name = "ldtteam"
        url = uri("https://ldtteam.jfrog.io/artifactory/parchmentmc-public/")
    }
}

dependencies {
	minecraft("net.minecraft", "minecraft", properties["minecraft_version"].toString())
	mappings(loom.layered {
		officialMojangMappings()
	})
	modImplementation("net.fabricmc", "fabric-loader", properties["loader_version"].toString())

	implementation(project(":core:api"))

	testImplementation("org.junit.jupiter:junit-jupiter-api:5.8.2")
	testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.8.2")
}

base {
	archivesBaseName = "blaze4d"
}

/*
java {
	sourceCompatibility = JavaVersion.VERSION_18
	targetCompatibility = JavaVersion.VERSION_18

	withSourcesJar()
}*/

loom {
	accessWidenerPath.set(file("src/main/resources/blaze4d.aw"))

	runs {
		val client by this
		client.vmArgs.add("--add-modules=jdk.incubator.foreign")
		client.vmArgs.add("--enable-native-access=ALL-UNNAMED") // should be graphics.kiln.blaze4d.core but modules are screwed
		client.vmArgs.add("-Db4d.native=" + property("b4d_native_path"))

		create("clientWithValidation") {
			inherit(client)
			configName = "Minecraft Client with Validation Layers"
			vmArgs.add("-Db4d.enable_validation")
		}

		create("clientWithRenderdoc") {
			inherit(client)
			configName = "Minecraft Client with Renderdoc"
			vmArgs.add("-Db4d.enable_renderdoc")
		}

		create("clientWithValidationRenderdoc") {
			inherit(client)
			configName = "Minecraft Client with Validation Layers and Renderdoc"
			vmArgs.add("-Db4d.enable_validation")
			vmArgs.add("-Db4d.enable_renderdoc")
		}
	}
}

tasks {
	test {
		useJUnitPlatform()
	}

	withType<JavaCompile> {
		options.encoding = "UTF-8"
		options.release.set(18)
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
