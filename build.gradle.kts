plugins {
    id("fabric-loom") version "0.9-SNAPSHOT"
    id("io.github.juuxel.loom-quiltflower") version "1.1.1"
    `maven-publish`
}

group = "me.hydos"
version = "1.0.0-SNAPSHOT"

repositories {
    mavenCentral()

    maven {
        name = "Sonatype Snapshots"
        url = uri("https://oss.sonatype.org/content/repositories/snapshots/")
    }
}

dependencies {
    minecraft("net.minecraft", "minecraft", properties["minecraft_version"].toString())
    mappings("net.fabricmc", "yarn", properties["yarn_mappings"].toString(), classifier = "v2")
    modImplementation("net.fabricmc", "fabric-loader", properties["loader_version"].toString())

    include(implementation(project(":Rosella"))!!)
    implementation(project(":Aftermath"))
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
