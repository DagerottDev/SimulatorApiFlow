// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SimulatorApiFlow",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(name: "SimulatorApiFlow", targets: ["SimulatorApiFlow"])
    ],
    targets: [
        .target(
            name: "SimulatorApiFlow",
            path: "Sources/SimulatorApiFlow"
        )
    ]
)
