plugins {
    `java-library`
}

group = "me.hydos"
version = "1.1.0"

val lwjglVersion = extra["lwjgl.version"].toString()
val lwjglNatives = extra["lwjgl.natives"].toString()

repositories {
    mavenCentral()
    maven("https://oss.sonatype.org/content/repositories/snapshots/")
}

dependencies {
    implementation(platform("org.lwjgl:lwjgl-bom:$lwjglVersion"))
    implementation("org.lwjgl", "lwjgl")
    implementation("org.jetbrains", "annotations", "20.1.0")
    runtimeOnly("org.lwjgl", "lwjgl", classifier = lwjglNatives)
}
