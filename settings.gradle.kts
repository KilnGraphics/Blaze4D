rootProject.name = "blaze4d"

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

include("mod", "core:api", "core:natives")
