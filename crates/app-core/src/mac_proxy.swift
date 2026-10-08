import Foundation
import SystemConfiguration
import Security

// Xcode is already required for Simulator capture. No host settings are changed by list/prepare.
func fail(_ message: String) throws -> Never { throw NSError(domain: "SimulatorApiFlow", code: 1, userInfo: [NSLocalizedDescriptionKey: message]) }
func captured(_ original: [String: Any]) -> [String: Any] {
    var result = original
    for prefix in ["HTTP", "HTTPS"] {
        result[prefix + "Enable"] = 1
        result[prefix + "Proxy"] = "127.0.0.1"
        result[prefix + "Port"] = 8181
    }
    return result
}
func equal(_ lhs: [String: Any], _ rhs: [String: Any]) -> Bool { NSDictionary(dictionary: lhs).isEqual(to: rhs) }
func conflicts(_ config: [String: Any]) -> Bool {
    ["HTTPEnable", "HTTPSEnable", "SOCKSEnable", "FTPEnable", "RTSPEnable", "GopherEnable", "ProxyAutoConfigEnable", "ProxyAutoDiscoveryEnable"].contains { (config[$0] as? NSNumber)?.boolValue == true }
    || config.keys.contains { $0.lowercased().contains("password") || $0.lowercased().contains("username") || $0.lowercased().contains("authentication") || ["HTTPUser", "HTTPSUser", "SOCKSUser", "FTPUser", "RTSPUser", "GopherUser"].contains($0) }
}
func usesCapture(_ settings: [String: Any]) -> Bool {
    if (settings["ProxyAutoConfigEnable"] as? NSNumber)?.boolValue == true || (settings["ProxyAutoDiscoveryEnable"] as? NSNumber)?.boolValue == true { return true }
    // Any enabled proxy on our listener port is conservatively treated as still owned.
    // This also covers aliases/custom DNS without resolving private proxy hostnames.
    return ["HTTP", "HTTPS", "SOCKS"].contains { prefix in
        (settings[prefix + "Enable"] as? NSNumber)?.boolValue == true
            && (settings[prefix + "Port"] as? NSNumber)?.intValue == 8181
    }
}
func effectiveProxies() throws -> [String: Any] {
    guard let settings = SCDynamicStoreCopyProxies(nil) as? [String: Any] else { try fail("Effective proxy settings are unavailable. Keep capture running and retry recovery.") }
    return settings
}
func primaryService() -> String? { (SCDynamicStoreCopyValue(nil, "State:/Network/Global/IPv4" as CFString) as? [String: Any])?["PrimaryService"] as? String }
func effectiveMatches(_ settings: [String: Any], _ target: [String: Any]) -> Bool {
    ["HTTP", "HTTPS"].allSatisfy { prefix in
        let enabled = (target[prefix + "Enable"] as? NSNumber)?.boolValue == true
        guard ((settings[prefix + "Enable"] as? NSNumber)?.boolValue == true) == enabled else { return false }
        return !enabled || ((settings[prefix + "Proxy"] as? String) == (target[prefix + "Proxy"] as? String) && (settings[prefix + "Port"] as? NSNumber) == (target[prefix + "Port"] as? NSNumber))
    }
}
func preferences(_ authorization: AuthorizationRef? = nil) throws -> SCPreferences {
    guard let prefs = authorization.map({ SCPreferencesCreateWithAuthorization(nil, "SimulatorApiFlow" as CFString, nil, $0) }) ?? SCPreferencesCreate(nil, "SimulatorApiFlow" as CFString, nil) else { try fail("Cannot read macOS network preferences.") }
    return prefs
}
func location(_ prefs: SCPreferences) throws -> SCNetworkSet {
    guard let set = SCNetworkSetCopyCurrent(prefs) else { try fail("No active Network Location.") }; return set
}
func proxyProtocol(_ prefs: SCPreferences, _ id: String) throws -> SCNetworkProtocol {
    let set = try location(prefs)
    let services = SCNetworkSetCopyServices(set) as? [SCNetworkService] ?? []
    guard let service = services.first(where: { (SCNetworkServiceGetServiceID($0) as String?) == id }), SCNetworkServiceGetEnabled(service), let proto = SCNetworkServiceCopyProtocol(service, kSCNetworkProtocolTypeProxies), SCNetworkProtocolGetEnabled(proto) else { try fail("The selected network service is no longer available or enabled.") }
    return proto
}
func config(_ proto: SCNetworkProtocol) -> [String: Any] { SCNetworkProtocolGetConfiguration(proto) as? [String: Any] ?? [:] }
func original(_ lease: [String: Any]) throws -> [String: Any] {
    guard let encoded = lease["original"] as? String, let data = Data(base64Encoded: encoded), data.count <= 65536, let value = try PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any] else { try fail("Invalid saved proxy configuration.") }; return value
}
func authorized() throws -> AuthorizationRef {
    var reference: AuthorizationRef?
    guard AuthorizationCreate(nil, nil, [], &reference) == errAuthorizationSuccess, let ref = reference else { try fail("macOS authorization is unavailable.") }
    let status = "system.preferences.network".withCString { name in
        var item = AuthorizationItem(name: name, valueLength: 0, value: nil, flags: 0)
        return withUnsafeMutablePointer(to: &item) { pointer in
            var rights = AuthorizationRights(count: 1, items: pointer)
            return AuthorizationCopyRights(ref, &rights, nil, [.interactionAllowed, .extendRights], nil)
        }
    }
    guard status == errAuthorizationSuccess else { AuthorizationFree(ref, []); try fail("Network authorization was declined. No proxy changes were applied.") }
    return ref
}
func run(_ input: [String: Any]) throws -> Any {
    let action = input["action"] as? String ?? ""
    if action == "selfcheck" {
        let unset: [String: Any] = [:]
        let previous: [String: Any] = ["HTTPEnable": 0, "HTTPProxy": "127.0.0.1", "HTTPPort": 9090, "ExceptionsList": ["*.local"]]
        assert(!conflicts(unset) && !conflicts(previous))
        assert(conflicts(["ProxyAutoConfigEnable": 1]) && conflicts(["SOCKSEnable": 1]))
        assert(equal(captured(unset), ["HTTPEnable": 1, "HTTPProxy": "127.0.0.1", "HTTPPort": 8181, "HTTPSEnable": 1, "HTTPSProxy": "127.0.0.1", "HTTPSPort": 8181]))
        assert((captured(previous)["ExceptionsList"] as? [String]) == ["*.local"])
        let encoded = try PropertyListSerialization.data(fromPropertyList: previous, format: .binary, options: 0).base64EncodedString()
        let decoded = try original(["original": encoded])
        assert(equal(decoded, previous))
        assert(!equal(captured(previous), ["HTTPProxy": "outside.example"]))
        assert(!usesCapture(previous) && usesCapture(captured(previous)))
        assert(usesCapture(["ProxyAutoConfigEnable": 1]))
        for host in ["localhost.", "0:0:0:0:0:0:0:1", "[::1]", "127.1", "::ffff:127.0.0.1", "custom.example"] {
            assert(usesCapture(["HTTPEnable": 1, "HTTPPort": 8181, "HTTPProxy": host]))
        }
        assert(!effectiveMatches([:], captured(previous)))
        assert(effectiveMatches([:], previous))
        assert(conflicts(["HTTPUser": "fixture-only"]))
        return ["passed": true]
    }
    guard ["list", "prepare", "enable", "disable", "canRelease"].contains(action) else { try fail("Unsupported proxy operation.") }
    let prefs = try preferences()
    let set = try location(prefs)
    if action == "list" {
        let primary = primaryService()
        return (SCNetworkSetCopyServices(set) as? [SCNetworkService] ?? []).filter { SCNetworkServiceGetEnabled($0) && SCNetworkServiceCopyProtocol($0, kSCNetworkProtocolTypeProxies) != nil }.map { service -> [String: Any] in
            let id = SCNetworkServiceGetServiceID(service) as String? ?? ""
            return ["id": id, "name": SCNetworkServiceGetName(service) as String? ?? id, "isDefault": id == primary]
        }
    }
    if action == "prepare" {
        guard let id = input["serviceId"] as? String else { try fail("Choose a Mac network service.") }
        guard primaryService() == id else { try fail("Select the current-route Mac network service for automatic Simulator capture.") }
        let proto = try proxyProtocol(prefs, id); let current = config(proto)
        guard !conflicts(current) else { try fail("This service already uses a proxy, PAC or proxy authentication. Its settings were left unchanged.") }
        let data = try PropertyListSerialization.data(fromPropertyList: current, format: .binary, options: 0)
        guard data.count <= 65536 else { try fail("The proxy configuration is too large.") }
        return ["serviceId": id, "locationId": SCNetworkSetGetSetID(set) as String? ?? "", "original": data.base64EncodedString(), "hadConfiguration": SCNetworkProtocolGetConfiguration(proto) != nil]
    }
    guard let lease = input["lease"] as? [String: Any], let id = lease["serviceId"] as? String, let locationId = lease["locationId"] as? String else { try fail("Missing saved proxy configuration.") }
    if action == "canRelease" {
        guard (SCNetworkSetGetSetID(set) as String?) == locationId else { return ["safe": false] }
        let proto = try proxyProtocol(prefs, id)
        let effective = try effectiveProxies()
        return ["safe": !usesCapture(config(proto)) && !usesCapture(effective)]
    }
    let before = try original(lease)
    let expected = captured(before)
    // Check before asking for authorization, and again inside the locked transaction.
    func checked(_ prefs: SCPreferences) throws -> SCNetworkProtocol {
        guard (SCNetworkSetGetSetID(try location(prefs)) as String?) == locationId else { try fail("Network Location changed. Return to the original location before restoring routing.") }
        if action == "enable" && primaryService() != id { try fail("The active network route changed. Select its network service before enabling routing.") }
        let proto = try proxyProtocol(prefs, id); let current = config(proto)
        let present = SCNetworkProtocolGetConfiguration(proto) != nil
        guard (present == (lease["hadConfiguration"] as? Bool) && equal(current, before)) || (present && equal(current, expected)) else { try fail("Proxy settings changed outside SimulatorApiFlow. They were preserved; review Network settings before recovering.") }
        return proto
    }
    let initial = try checked(prefs)
    let target = action == "enable" ? expected : before
    // Avoid another authorization dialog when recovery is already complete. A committed but
    // not applied configuration must still go through ApplyChanges below.
    if action == "disable", equal(config(initial), before),
       (SCNetworkProtocolGetConfiguration(initial) != nil) == (lease["hadConfiguration"] as? Bool) {
        let effective = try effectiveProxies()
        if !usesCapture(effective) && (primaryService() != id || effectiveMatches(effective, before)) {
            return ["enabled": false]
        }
    }
    let auth = try authorized(); defer { AuthorizationFree(auth, []) }
    let writable = try preferences(auth)
    guard SCPreferencesLock(writable, false) else { try fail("Cannot lock network preferences. Retry after other network settings finish.") }
    defer { SCPreferencesUnlock(writable) }
    let proto = try checked(writable)
    let dictionary: CFDictionary? = action == "disable" && (lease["hadConfiguration"] as? Bool) == false ? nil : target as CFDictionary
    guard SCNetworkProtocolSetConfiguration(proto, dictionary), SCPreferencesCommitChanges(writable) else { try fail("Could not commit proxy settings. Saved recovery information remains available.") }
    guard SCPreferencesApplyChanges(writable) else { try fail("Proxy settings were committed but could not be applied. Keep capture running and retry recovery.") }
    let verified = try preferences(); let final = try checked(verified)
    guard equal(config(final), target), (SCNetworkProtocolGetConfiguration(final) != nil) == (action == "enable" || (lease["hadConfiguration"] as? Bool) == true) else { try fail("Proxy settings could not be verified. Keep capture running and retry recovery.") }
    var applied = false
    for _ in 0..<50 {
        let primary = primaryService()
        let effective = try effectiveProxies()
        if action == "enable" {
            guard primary == id else { try fail("The active network route changed during setup. Keep capture running and retry routing recovery.") }
            applied = effectiveMatches(effective, target)
        } else {
            applied = !usesCapture(effective) && (primary != id || effectiveMatches(effective, target))
        }
        if applied { break }
        Thread.sleep(forTimeInterval: 0.1)
    }
    guard applied else { try fail("Effective network routing could not be verified. Keep capture running and retry recovery.") }
    return ["enabled": action == "enable"]
}
do {
    let data = FileHandle.standardInput.readDataToEndOfFile()
    guard data.count <= 131072, let input = try JSONSerialization.jsonObject(with: data) as? [String: Any] else { try fail("Invalid proxy operation.") }
    let result = try run(input)
    FileHandle.standardOutput.write(try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys]))
} catch {
    // Never log saved proxy configuration, command payloads or authorization material.
    FileHandle.standardError.write(Data((error.localizedDescription + "\n").utf8))
    exit(1)
}
