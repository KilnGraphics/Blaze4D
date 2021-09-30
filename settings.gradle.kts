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
