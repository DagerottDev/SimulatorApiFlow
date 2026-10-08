import Foundation
#if os(iOS)
import UIKit
#endif

public enum SimulatorApiFlow {
    public static let sdkVersion = "0.1.0"
    public static let correlationHeader = "X-Mobile-API-Studio-Request-Id"

    public static var defaultEnabled: Bool {
        #if DEBUG
        true
        #else
        false
        #endif
    }

    private static let state = SimulatorApiFlowState()

    internal static var isEnabled: Bool { state.isEnabled }

    public static func configure(_ configuration: SimulatorApiFlowConfiguration = .init()) {
        state.configure(configuration)
    }

    public static func disable() {
        var configuration = SimulatorApiFlowConfiguration()
        configuration.enabled = false
        state.configure(configuration)
    }

    public static func setContext(
        screen: String? = nil,
        feature: String? = nil,
        attributes: [String: String] = [:],
        file: String = #fileID,
        function: String = #function,
        line: UInt = #line
    ) {
        state.setContext(
            SimulatorApiFlowContext(
                screen: screen,
                feature: feature,
                attributes: attributes,
                source: SimulatorApiFlowSource(file: file, function: function, line: line)
            )
        )
    }

    public static func log(
        _ message: String,
        level: SimulatorApiFlowLogLevel = .info,
        attributes: [String: String] = [:],
        file: String = #fileID,
        function: String = #function,
        line: UInt = #line
    ) {
        state.log(
            message,
            level: level,
            attributes: attributes,
            source: SimulatorApiFlowSource(file: file, function: function, line: line)
        )
    }

    @discardableResult
    public static func instrument(
        _ request: inout URLRequest,
        feature: String? = nil,
        attributes: [String: String] = [:],
        file: String = #fileID,
        function: String = #function,
        line: UInt = #line
    ) -> String? {
        state.instrument(
            &request,
            feature: feature,
            attributes: attributes,
            source: SimulatorApiFlowSource(file: file, function: function, line: line)
        )
    }

    public static func instrumented(
        _ request: URLRequest,
        feature: String? = nil,
        attributes: [String: String] = [:],
        file: String = #fileID,
        function: String = #function,
        line: UInt = #line
    ) -> SimulatorApiFlowInstrumentedRequest? {
        var copy = request
        guard let requestID = instrument(
            &copy,
            feature: feature,
            attributes: attributes,
            file: file,
            function: function,
            line: line
        ) else {
            return nil
        }
        return SimulatorApiFlowInstrumentedRequest(requestID: requestID, request: copy)
    }

    public static func complete(
        requestID: String,
        response: URLResponse? = nil,
        error: Error? = nil
    ) {
        state.complete(requestID: requestID, response: response, error: error)
    }

    public static func instrument(_ configuration: URLSessionConfiguration) -> URLSessionConfiguration {
        guard state.isEnabled else { return configuration }
        var classes = configuration.protocolClasses ?? []
        if !classes.contains(where: { $0 == SimulatorApiFlowURLProtocol.self }) {
            classes.insert(SimulatorApiFlowURLProtocol.self, at: 0)
            configuration.protocolClasses = classes
        }
        return configuration
    }

    internal static func autoInstrument(_ request: URLRequest) -> SimulatorApiFlowInstrumentedRequest? {
        var copy = request
        guard let requestID = state.instrument(
            &copy,
            feature: nil,
            attributes: [:],
            source: nil
        ) else { return nil }
        return SimulatorApiFlowInstrumentedRequest(requestID: requestID, request: copy)
    }
}

private final class SimulatorApiFlowState: @unchecked Sendable {
    private struct InFlight {
        var startedAt: Date
        var method: String
        var url: String
        var context: SimulatorApiFlowContext
    }

    private let lock = NSLock()
    private var configuration: SimulatorApiFlowConfiguration?
    private var transport: SimulatorApiFlowTransport?
    private var currentContext = SimulatorApiFlowContext()
    private var inFlight: [String: InFlight] = [:]
    private var clientID: String?

    var isEnabled: Bool {
        lock.withLock { configuration?.enabled == true }
    }

    func configure(_ configuration: SimulatorApiFlowConfiguration) {
        lock.withLock {
            self.configuration = configuration
            guard configuration.enabled else {
                transport = nil
                clientID = nil
                inFlight.removeAll()
                return
            }
            transport = SimulatorApiFlowTransport(baseURL: configuration.desktopBaseURL, pairingToken: configuration.pairingToken)
            clientID = loadClientID(appID: configuration.appID)
        }
        sendHandshake()
    }

    func setContext(_ context: SimulatorApiFlowContext) {
        let payload: [String: Any]? = lock.withLock {
            currentContext = context
            guard let clientID, configuration?.enabled == true else { return nil }
            return ["clientId": clientID, "context": context.wireValue]
        }
        if let payload { send(type: "context", payload: payload) }
    }

    func log(
        _ message: String,
        level: SimulatorApiFlowLogLevel,
        attributes: [String: String],
        source: SimulatorApiFlowSource
    ) {
        let payload: [String: Any]? = lock.withLock {
            guard let clientID, configuration?.enabled == true else { return nil }
            var context = currentContext
            context.attributes.merge(attributes) { _, new in new }
            context.source = source
            return [
                "clientId": clientID,
                "level": level.rawValue,
                "message": message,
                "context": context.wireValue
            ]
        }
        if let payload { send(type: "log", payload: payload) }
    }

    func instrument(
        _ request: inout URLRequest,
        feature: String?,
        attributes: [String: String],
        source: SimulatorApiFlowSource?
    ) -> String? {
        let result: (String, [String: Any])? = lock.withLock {
            guard let clientID, configuration?.enabled == true,
                  let url = request.url else { return nil }
            let requestID = UUID().uuidString.lowercased()
            request.setValue(requestID, forHTTPHeaderField: SimulatorApiFlow.correlationHeader)
            var context = currentContext
            if let feature { context.feature = feature }
            context.attributes.merge(attributes) { _, new in new }
            if let source { context.source = source }
            let method = request.httpMethod ?? "GET"
            inFlight[requestID] = InFlight(
                startedAt: Date(),
                method: method,
                url: url.absoluteString,
                context: context
            )
            return (
                requestID,
                [
                    "clientId": clientID,
                    "requestId": requestID,
                    "phase": "started",
                    "method": method,
                    "url": url.absoluteString,
                    "context": context.wireValue
                ]
            )
        }
        guard let result else { return nil }
        send(type: "network", payload: result.1)
        return result.0
    }

    func complete(requestID: String, response: URLResponse?, error: Error?) {
        let payload: [String: Any]? = lock.withLock {
            guard let clientID,
                  configuration?.enabled == true,
                  let flight = inFlight.removeValue(forKey: requestID) else { return nil }
            var value: [String: Any] = [
                "clientId": clientID,
                "requestId": requestID,
                "phase": error == nil ? "completed" : "failed",
                "method": flight.method,
                "url": flight.url,
                "durationMs": Int(Date().timeIntervalSince(flight.startedAt) * 1000),
                "context": flight.context.wireValue
            ]
            if let response = response as? HTTPURLResponse {
                value["statusCode"] = response.statusCode
            }
            if let error { value["error"] = String(describing: error) }
            return value
        }
        if let payload { send(type: "network", payload: payload) }
    }

    private func sendHandshake() {
        let payload: [String: Any]? = lock.withLock {
            guard let configuration,
                  configuration.enabled,
                  let clientID else { return nil }
            var value: [String: Any] = [
                "clientId": clientID,
                "appId": configuration.appID,
                "appName": configuration.appName,
                "platform": "ios",
                "sdkVersion": SimulatorApiFlow.sdkVersion
            ]
            if let appVersion = configuration.appVersion { value["appVersion"] = appVersion }
            if let appBuild = configuration.appBuild { value["appBuild"] = appBuild }
            #if os(iOS)
            value["deviceName"] = UIDevice.current.name
            value["osVersion"] = UIDevice.current.systemVersion
            #else
            value["deviceName"] = Host.current().localizedName
            value["osVersion"] = ProcessInfo.processInfo.operatingSystemVersionString
            #endif
            return value
        }
        if let payload { send(type: "handshake", payload: payload) }
    }

    private func send(type: String, payload: [String: Any]) {
        let currentTransport = lock.withLock { transport }
        currentTransport?.send(type: type, payload: payload)
    }

    private func loadClientID(appID: String) -> String {
        let key = "dev.simulatorapiflow.client-id.\(appID)"
        if let existing = UserDefaults.standard.string(forKey: key), !existing.isEmpty {
            return existing
        }
        let generated = UUID().uuidString.lowercased()
        UserDefaults.standard.set(generated, forKey: key)
        return generated
    }
}

private extension NSLock {
    func withLock<T>(_ body: () -> T) -> T {
        lock()
        defer { unlock() }
        return body()
    }
}
