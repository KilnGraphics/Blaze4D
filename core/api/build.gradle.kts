plugins {
    id("java")
    id("fr.stardustenterprises.rust.importer") version "3.2.4"
}

group = "graphics.kiln"
version = "1.0.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    rust(project(":core:natives"))

    implementation("org.apache.logging.log4j:log4j-api:2.17.0")
    implementation("org.apache.commons:commons-lang3:3.12.0")
    implementation("com.google.code.gson:gson:2.8.9")
    implementation("org.lwjgl:lwjgl-glfw:3.3.1")

    testImplementation("org.junit.jupiter:junit-jupiter-api:5.8.2")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.8.2")
}

rustImport {
    baseDir.set("/graphics/kiln/blaze4d/core/natives")
    layout.set("hierarchical")
}

tasks.withType<JavaCompile> {
    options.release.set(18)
    options.compilerArgs.add("--add-modules=jdk.incubator.foreign")
}

tasks.getByName<Test>("test") {
    useJUnitPlatform()
}