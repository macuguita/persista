plugins {
	alias(libs.plugins.kotlin.jvm)
	alias(libs.plugins.kotlin.serialization)
	alias(libs.plugins.shadow)
	alias(libs.plugins.spotless)
    application
}

group = "com.macuguita"
version = "1.0.0"

repositories {
    mavenCentral()
}

application {
    mainClass.set("com.macuguita.persista.ApplicationKt")
}

dependencies {
    implementation(libs.bundles.ktor.server)
    implementation(libs.bundles.ktor.client)
    implementation(libs.bundles.exposed)
    implementation(libs.postgresql)
    implementation(libs.hikari)
    implementation(libs.java.jwt)
    implementation(libs.logback)
    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(25)
}

spotless {
	lineEndings = com.diffplug.spotless.LineEnding.UNIX
	kotlin {
		ktlint()
	}
}

tasks.jar {
    manifest {
        attributes(
            "Main-Class" to application.mainClass.get()
        )
    }
}

tasks.shadowJar {
    archiveClassifier.set("all")
}

tasks.test {
    useJUnitPlatform()
}
