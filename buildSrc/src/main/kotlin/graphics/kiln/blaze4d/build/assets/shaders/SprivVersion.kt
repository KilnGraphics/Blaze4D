package graphics.kiln.blaze4d.build.assets.shaders

enum class SprivVersion(val cliArg: String) {
    SPV_1_0("--target-spv=spv1.0"),
    SPV_1_1("--target-spv=spv1.1"),
    SPV_1_2("--target-spv=spv1.2"),
    SPV_1_3("--target-spv=spv1.3"),
    SPV_1_4("--target-spv=spv1.4"),
    SPV_1_5("--target-spv=spv1.5"),
    SPV_1_6("--target-spv=spv1.6"),
}