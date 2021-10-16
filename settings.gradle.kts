pluginManagement {
    repositories {
        gradlePluginPortal()

        maven {
            name = "FabricMC"
            url = uri("https://maven.fabricmc.net/")
        }

        maven {
            name = "Cotton"
            url = uri("https://server.bbkr.space/artifactory/libs-release/")
        }
    }
}

// If the user is running an in-tree copy of Rosella, add that as a sub-project
val rosellaDir = File("Rosella")
if (rosellaDir.exists()) {
    include(":rosella")
    project(":rosella").projectDir = rosellaDir
}
