plugins {
    id("com.android.application")
}

android {
    namespace = "dev.simulatorapiflow.sample"
    compileSdk = 37

    defaultConfig {
        applicationId = "dev.simulatorapiflow.sample"
        minSdk = 23
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"
    }
}

dependencies {
    implementation(project(":simulator-api-flow"))
    implementation("com.squareup.okhttp3:okhttp:5.4.0")
}
