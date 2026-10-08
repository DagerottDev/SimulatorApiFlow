import SimulatorApiFlow
import SwiftUI

@main
struct SimulatorApiFlowSampleApp: App {
    init() {
        SimulatorApiFlow.configure()
        SimulatorApiFlow.setContext(
            screen: "Sample Home",
            feature: "SDK Demo",
            attributes: ["platform": "ios"]
        )
        SimulatorApiFlow.log("iOS sample launched")
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
