import Foundation
import Carbon
import AppKit

// MARK: - Debug Logging

// Only log when /tmp/gonhanh_debug.log exists (touch /tmp/gonhanh_debug.log to enable)
func debugLog(_ message: String) {
    let logPath = "/tmp/gonhanh_debug.log"
    guard FileManager.default.fileExists(atPath: logPath) else { return }

    let timestamp = ISO8601DateFormatter().string(from: Date())
    let logMessage = "[\(timestamp)] \(message)\n"

    if let handle = FileHandle(forWritingAtPath: logPath) {
        handle.seekToEndOfFile()
        if let data = logMessage.data(using: .utf8) {
            handle.write(data)
        }
        handle.closeFile()
    }
}

// MARK: - FFI Result Struct (must match Rust #[repr(C)])

struct ImeResult {
    var chars: (UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32,
                UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32)
    var action: UInt8      // 0=None, 1=Send, 2=Restore
    var backspace: UInt8
    var count: UInt8
    var _pad: UInt8
}

// MARK: - C Function Declarations

@_silgen_name("ime_init")
func ime_init()

@_silgen_name("ime_key")
func ime_key(_ key: UInt16, _ caps: Bool, _ ctrl: Bool) -> UnsafeMutablePointer<ImeResult>?

@_silgen_name("ime_key_ext")
func ime_key_ext(_ key: UInt16, _ caps: Bool, _ ctrl: Bool, _ shift: Bool) -> UnsafeMutablePointer<ImeResult>?

@_silgen_name("ime_method")
func ime_method(_ method: UInt8)

@_silgen_name("ime_enabled")
func ime_enabled(_ enabled: Bool)

@_silgen_name("ime_clear")
func ime_clear()

@_silgen_name("ime_free")
func ime_free(_ result: UnsafeMutablePointer<ImeResult>?)

// MARK: - RustBridge

class RustBridge {
    static var isInitialized = false

    /// Initialize engine (call once at app start)
    static func initialize() {
        guard !isInitialized else { return }
        ime_init()
        isInitialized = true
        debugLog("[RustBridge] Engine initialized")
    }

    /// Process key event
    /// Returns: (backspaceCount, newChars) or nil if no action needed
    /// - Parameters:
    ///   - keyCode: macOS virtual keycode
    ///   - caps: true if CapsLock is active (for uppercase letters)
    ///   - ctrl: true if Cmd/Ctrl/Alt is pressed (bypasses IME)
    ///   - shift: true if Shift key is pressed (for symbols like @, #, $)
    static func processKey(keyCode: UInt16, caps: Bool, ctrl: Bool, shift: Bool = false) -> (Int, [Character])? {
        guard isInitialized else {
            debugLog("[RustBridge] Engine not initialized!")
            return nil
        }

        guard let resultPtr = ime_key_ext(keyCode, caps, ctrl, shift) else {
            return nil
        }
        defer { ime_free(resultPtr) }

        let result = resultPtr.pointee

        // Action: 0=None, 1=Send, 2=Restore
        guard result.action == 1 else {
            return nil
        }

        let backspace = Int(result.backspace)
        var chars: [Character] = []

        // Extract chars from tuple
        let charArray = withUnsafePointer(to: result.chars) { ptr in
            ptr.withMemoryRebound(to: UInt32.self, capacity: 32) { bound in
                Array(UnsafeBufferPointer(start: bound, count: Int(result.count)))
            }
        }

        for code in charArray {
            if let scalar = Unicode.Scalar(code) {
                chars.append(Character(scalar))
            }
        }

        return (backspace, chars)
    }

    /// Set input method (0=Telex, 1=VNI)
    static func setMethod(_ method: Int) {
        ime_method(UInt8(method))
        debugLog("[RustBridge] Method set to: \(method == 0 ? "Telex" : "VNI")")
    }

    /// Enable/disable engine
    static func setEnabled(_ enabled: Bool) {
        ime_enabled(enabled)
        debugLog("[RustBridge] Engine enabled: \(enabled)")
    }

    /// Clear buffer (new session, e.g., on mouse click)
    static func clearBuffer() {
        ime_clear()
    }
}

// MARK: - Keyboard Hook Manager

class KeyboardHookManager {
    static let shared = KeyboardHookManager()

    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var isRunning = false

    private init() {}

    func start() {
        guard !isRunning else { return }

        debugLog("[KeyboardHook] Starting...")

        // Check accessibility permission
        let trusted = AXIsProcessTrusted()
        debugLog("[KeyboardHook] Accessibility trusted: \(trusted)")

        if !trusted {
            // Prompt user for permission
            let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
            AXIsProcessTrustedWithOptions(options)
            debugLog("[KeyboardHook] Requested accessibility permission. Please grant and restart app.")
            return
        }

        // Initialize Rust engine
        RustBridge.initialize()

        // Create event tap for keyDown events
        // Use listenOnly option which doesn't require as strict permissions
        let eventMask: CGEventMask = (1 << CGEventType.keyDown.rawValue)

        debugLog("[KeyboardHook] Creating event tap...")

        // Try creating tap - use .cghidEventTap for better compatibility
        var tap: CFMachPort?

        // First try session tap with defaultTap (can modify events)
        tap = CGEvent.tapCreate(
            tap: .cghidEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: eventMask,
            callback: keyboardCallback,
            userInfo: nil
        )

        if tap == nil {
            debugLog("[KeyboardHook] cghidEventTap failed, trying cgSessionEventTap...")
            tap = CGEvent.tapCreate(
                tap: .cgSessionEventTap,
                place: .headInsertEventTap,
                options: .defaultTap,
                eventsOfInterest: eventMask,
                callback: keyboardCallback,
                userInfo: nil
            )
        }

        if tap == nil {
            debugLog("[KeyboardHook] cgSessionEventTap failed, trying cgAnnotatedSessionEventTap...")
            tap = CGEvent.tapCreate(
                tap: .cgAnnotatedSessionEventTap,
                place: .headInsertEventTap,
                options: .defaultTap,
                eventsOfInterest: eventMask,
                callback: keyboardCallback,
                userInfo: nil
            )
        }

        guard let finalTap = tap else {
            debugLog("[KeyboardHook] ALL event tap methods FAILED!")
            debugLog("[KeyboardHook] Opening System Settings for Input Monitoring...")

            // Show alert and open System Settings
            DispatchQueue.main.async {
                let alert = NSAlert()
                alert.messageText = "Cần quyền Accessibility"
                alert.informativeText = "GoNhanh cần quyền Accessibility để gõ tiếng Việt.\n\n1. Mở System Settings > Privacy & Security > Accessibility\n2. Bật GoNhanh\n3. Khởi động lại app"
                alert.alertStyle = .warning
                alert.addButton(withTitle: "Mở System Settings")
                alert.addButton(withTitle: "Hủy")

                if alert.runModal() == .alertFirstButtonReturn {
                    // Open Accessibility settings
                    if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                        NSWorkspace.shared.open(url)
                    }
                }
            }
            return
        }

        debugLog("[KeyboardHook] Event tap created successfully")

        eventTap = finalTap
        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, finalTap, 0)

        if let source = runLoopSource {
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
            CGEvent.tapEnable(tap: finalTap, enable: true)
            isRunning = true
            debugLog("[KeyboardHook] Started successfully, listening for keys...")
        }
    }

    func stop() {
        guard isRunning else { return }

        if let tap = eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
        }

        if let source = runLoopSource {
            CFRunLoopRemoveSource(CFRunLoopGetCurrent(), source, .commonModes)
        }

        eventTap = nil
        runLoopSource = nil
        isRunning = false
        debugLog("[KeyboardHook] Stopped")
    }

    func getTap() -> CFMachPort? {
        return eventTap
    }
}

// MARK: - Global Hotkey

enum GlobalHotkey {
    static let toggleKey: CGKeyCode = 0x31 // Space

    static func isToggleHotkey(_ keyCode: UInt16, _ flags: CGEventFlags) -> Bool {
        keyCode == toggleKey && flags.contains(.maskControl) && !flags.contains(.maskCommand)
    }
}

// MARK: - Keyboard Callback

// Magic number to identify events generated by GoNhanh
private let kGoNhanhEventMarker: Int64 = 0x474E4820  // "GNH " in hex

private func keyboardCallback(
    proxy: CGEventTapProxy,
    type: CGEventType,
    event: CGEvent,
    refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {

    // Handle tap disabled event - re-enable
    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        debugLog("[KeyboardHook] Event tap was disabled, re-enabling...")
        if let tap = KeyboardHookManager.shared.getTap() {
            CGEvent.tapEnable(tap: tap, enable: true)
        }
        return Unmanaged.passUnretained(event)
    }

    // Only handle key down
    guard type == .keyDown else {
        return Unmanaged.passUnretained(event)
    }

    // Skip events generated by GoNhanh (check userData marker)
    let userData = event.getIntegerValueField(.eventSourceUserData)
    if userData == kGoNhanhEventMarker {
        debugLog("[KeyboardHook] Skipping self-generated event")
        return Unmanaged.passUnretained(event)
    }

    let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    let flags = event.flags

    // Global hotkey: Ctrl+Space to toggle Vietnamese/English
    if GlobalHotkey.isToggleHotkey(keyCode, flags) {
        DispatchQueue.main.async {
            NotificationCenter.default.post(name: .toggleVietnamese, object: nil)
        }
        return nil // Consume the event
    }

    // Separate shift from caps for VNI Shift+number handling
    let shift = flags.contains(.maskShift)
    let caps = shift || flags.contains(.maskAlphaShift)
    let ctrl = flags.contains(.maskCommand) || flags.contains(.maskControl) ||
               flags.contains(.maskAlternate)

    debugLog("[KeyboardHook] Key: \(keyCode), caps=\(caps), shift=\(shift), ctrl=\(ctrl)")

    // Process key through Rust engine
    if let (backspace, chars) = RustBridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
        let charsStr = String(chars)
        debugLog("[KeyboardHook] Rust returned: backspace=\(backspace), chars=\"\(charsStr)\" (count=\(chars.count))")

        // Smart text replacement based on focused element detection
        // - Default: Backspace (fast, no flicker)
        // - AXComboBox/AXSearchField: Selection (fixes "dính chữ")
        sendTextReplacement(backspaceCount: backspace, chars: chars)

        // Consume original event
        debugLog("[KeyboardHook] Original event CONSUMED (returning nil)")
        return nil
    }

    // Pass through
    debugLog("[KeyboardHook] Pass through (no transform)")
    return Unmanaged.passUnretained(event)
}

// MARK: - Focused Element Detection

/// Information about the currently focused UI element
private struct FocusedElementInfo {
    let role: String?
    let subrole: String?
    let roleDescription: String?
    let bundleId: String
}

/// Get information about the currently focused UI element using Accessibility API
private func getFocusedElementInfo() -> FocusedElementInfo? {
    guard let frontApp = NSWorkspace.shared.frontmostApplication else {
        return nil
    }

    let bundleId = frontApp.bundleIdentifier ?? ""

    // Get system-wide accessibility element
    let systemWide = AXUIElementCreateSystemWide()

    // Get focused element
    var focusedElement: CFTypeRef?
    let focusError = AXUIElementCopyAttributeValue(
        systemWide,
        kAXFocusedUIElementAttribute as CFString,
        &focusedElement
    )

    guard focusError == .success, let element = focusedElement else {
        debugLog("[AX] Failed to get focused element: \(focusError.rawValue), bundleId=\(bundleId)")
        return FocusedElementInfo(role: nil, subrole: nil, roleDescription: nil, bundleId: bundleId)
    }

    // Get role
    var roleValue: CFTypeRef?
    AXUIElementCopyAttributeValue(
        element as! AXUIElement,
        kAXRoleAttribute as CFString,
        &roleValue
    )
    let role = roleValue as? String

    // Get subrole
    var subroleValue: CFTypeRef?
    AXUIElementCopyAttributeValue(
        element as! AXUIElement,
        kAXSubroleAttribute as CFString,
        &subroleValue
    )
    let subrole = subroleValue as? String

    // Get role description (more human-readable)
    var roleDescValue: CFTypeRef?
    AXUIElementCopyAttributeValue(
        element as! AXUIElement,
        kAXRoleDescriptionAttribute as CFString,
        &roleDescValue
    )
    let roleDesc = roleDescValue as? String

    debugLog("[AX] Focused: role=\(role ?? "nil"), subrole=\(subrole ?? "nil"), desc=\(roleDesc ?? "nil"), app=\(bundleId)")

    return FocusedElementInfo(role: role, subrole: subrole, roleDescription: roleDesc, bundleId: bundleId)
}

/// Replacement method type
private enum ReplacementMethod {
    case backspace      // Default: fast, no flicker
    case backspaceSlow  // For terminal apps: needs longer delays
    case selection      // For autocomplete fields: prevents "dính chữ"
}

/// Determine the best text replacement method based on focused element
///
/// Strategy:
/// 1. AXComboBox → Selection (address bars, dropdowns)
/// 2. AXTextField in Chrome/Safari/Arc → Selection (address bar uses AXTextField)
/// 3. AXSearchField → Selection (search boxes with autocomplete)
/// 4. JetBrains IDEs → Selection (code autocomplete)
/// 5. Microsoft Excel → Selection (cell autocomplete)
/// 6. Everything else → Backspace (default, ~90% of cases)
private func getReplacementMethod() -> ReplacementMethod {
    guard let info = getFocusedElementInfo() else {
        return .backspace // Default if can't detect
    }

    // Rule 1: ComboBox = address bar, dropdown → always Selection
    if info.role == "AXComboBox" {
        debugLog("[Method] AXComboBox detected → Selection")
        return .selection
    }

    // Rule 2: Chrome/Safari/Arc address bar (AXTextField with autocomplete)
    // Chrome uses AXTextField for omnibox, not AXComboBox
    let browserBundles = ["com.google.Chrome", "com.apple.Safari", "company.thebrowser.Browser"]
    if browserBundles.contains(info.bundleId) {
        // In browsers, AXTextField is likely the address bar (has autocomplete)
        // AXTextArea or AXWebArea is the page content (no autocomplete issue)
        if info.role == "AXTextField" {
            debugLog("[Method] Browser AXTextField (address bar) → Selection")
            return .selection
        }
    }

    // Rule 3: Search field with autocomplete → Selection
    if info.role == "AXTextField" && info.subrole == "AXSearchField" {
        debugLog("[Method] AXSearchField detected → Selection")
        return .selection
    }

    // Rule 4: JetBrains IDEs (IntelliJ, WebStorm, etc.) → Selection
    if info.bundleId.hasPrefix("com.jetbrains") {
        debugLog("[Method] JetBrains IDE detected → Selection")
        return .selection
    }

    // Rule 5: Microsoft Excel → Selection (cell autocomplete)
    if info.bundleId == "com.microsoft.Excel" {
        debugLog("[Method] Excel detected → Selection")
        return .selection
    }

    // Rule 6: Microsoft Word → Selection (suggestion popup)
    if info.bundleId == "com.microsoft.Word" {
        debugLog("[Method] Word detected → Selection")
        return .selection
    }

    // Rule 7: VS Code / Cursor / Terminal apps - needs longer delays
    // These apps don't expose AX focused element properly and have slower event processing
    let terminalApps = [
        "com.microsoft.VSCode",
        "com.todesktop.230313mzl4w4u92",  // Cursor
        "com.apple.Terminal",
        "com.googlecode.iterm2",
        "io.alacritty",
        "com.github.wez.wezterm",
        "com.google.antigravity",         // Antigravity (Claude Code)
        "dev.warp.Warp-Stable"            // Warp terminal
    ]
    if terminalApps.contains(info.bundleId) {
        debugLog("[Method] Terminal app detected → Backspace (slow)")
        return .backspaceSlow
    }

    // Default: Backspace (fast, no flicker)
    debugLog("[Method] Default → Backspace")
    return .backspace
}

// MARK: - Key Codes

private enum KeyCode {
    static let backspace: CGKeyCode = 0x33
    static let leftArrow: CGKeyCode = 0x7B
}

// MARK: - Send Keys

/// Smart text replacement - uses different methods based on focused element type
///
/// Strategy:
/// - Default: Backspace (fast, no flicker, works for ~90% of cases)
/// - Autocomplete contexts: Selection (prevents "dính chữ" in address bars, Excel, etc.)
///
/// Detection is done via Accessibility API to check the focused element's role,
/// which is more accurate than app-based detection.
private func sendTextReplacement(backspaceCount: Int, chars: [Character]) {
    // Run synchronously to ensure events are sent before callback returns
    // This prevents race condition where next key arrives before backspace is processed
    let method = getReplacementMethod()

    switch method {
    case .selection:
        sendTextReplacementWithSelection(backspaceCount: backspaceCount, chars: chars)
    case .backspace:
        sendTextReplacementWithBackspace(backspaceCount: backspaceCount, chars: chars, slow: false)
    case .backspaceSlow:
        sendTextReplacementWithBackspace(backspaceCount: backspaceCount, chars: chars, slow: true)
    }
}

/// Default method: backspace then type
/// - slow: true for terminal apps that need longer delays
private func sendTextReplacementWithBackspace(backspaceCount: Int, chars: [Character], slow: Bool) {
    let string = String(chars)
    let mode = slow ? "SLOW" : "FAST"
    debugLog("[Send:BS:\(mode)] START - backspace=\(backspaceCount), chars=\"\(string)\" (len=\(chars.count))")

    // Use privateState for all - the key difference is timing, not event source
    guard let source = CGEventSource(stateID: .privateState) else {
        debugLog("[Send:BS:\(mode)] FAILED - Cannot create CGEventSource")
        return
    }
    debugLog("[Send:BS:\(mode)] CGEventSource created OK")

    // For terminal apps: use longer delays but same tap as regular apps
    // The key is marking events to avoid re-processing, not changing the tap type
    if slow {
        // Send all backspaces first
        for i in 0..<backspaceCount {
            guard let down = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.backspace, keyDown: true),
                  let up = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.backspace, keyDown: false) else {
                debugLog("[Send:BS:\(mode)] FAILED - Cannot create backspace event \(i)")
                continue
            }
            // Mark as self-generated to avoid re-processing
            down.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
            down.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
            usleep(1500) // 1.5ms between backspaces for terminal
            debugLog("[Send:BS:\(mode)] Backspace \(i+1)/\(backspaceCount) sent")
        }

        // Wait for backspaces to be processed by terminal
        if backspaceCount > 0 {
            usleep(3000) // 3ms
        }

        // Send unicode
        let utf16 = Array(string.utf16)
        debugLog("[Send:BS:\(mode)] Sending unicode: \(utf16.map { String(format: "0x%04X", $0) }.joined(separator: " "))")
        guard let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
              let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else {
            debugLog("[Send:BS:\(mode)] FAILED - Cannot create unicode event")
            return
        }
        // Mark as self-generated
        down.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
        up.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
        down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
        up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
        down.post(tap: .cgSessionEventTap)
        up.post(tap: .cgSessionEventTap)
        usleep(2000) // 2ms after
        debugLog("[Send:BS:\(mode)] DONE")
        return
    }

    // Fast path for regular apps
    for i in 0..<backspaceCount {
        guard let down = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.backspace, keyDown: true),
              let up = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.backspace, keyDown: false) else {
            continue
        }
        // Mark as self-generated
        down.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
        up.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
        down.post(tap: .cgSessionEventTap)
        up.post(tap: .cgSessionEventTap)
        if i < backspaceCount - 1 {
            usleep(200)
        }
    }

    if backspaceCount > 0 {
        usleep(800)
    }

    let utf16 = Array(string.utf16)
    guard let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else {
        return
    }
    // Mark as self-generated
    down.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
    up.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
    down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    down.post(tap: .cgSessionEventTap)
    up.post(tap: .cgSessionEventTap)
    usleep(500)
    debugLog("[Send:BS:\(mode)] DONE")
}

/// GUI app-friendly: select then replace (atomic, fixes Chrome/Excel autocomplete)
private func sendTextReplacementWithSelection(backspaceCount: Int, chars: [Character]) {
    guard let source = CGEventSource(stateID: .privateState) else {
        debugLog("[Send] Failed to create CGEventSource")
        return
    }

    if backspaceCount > 0 {
        // Select text with Shift+Left Arrow
        for i in 0..<backspaceCount {
            guard let down = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.leftArrow, keyDown: true),
                  let up = CGEvent(keyboardEventSource: source, virtualKey: KeyCode.leftArrow, keyDown: false) else {
                debugLog("[Send] Failed to create shift+left event \(i)")
                continue
            }
            // Mark as self-generated
            down.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
            up.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
            down.flags = .maskShift
            up.flags = .maskShift
            down.post(tap: .cgSessionEventTap)
            up.post(tap: .cgSessionEventTap)
        }
    }

    // Send replacement characters (replaces selection)
    let string = String(chars)
    let utf16 = Array(string.utf16)

    guard let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else {
        debugLog("[Send] Failed to create unicode event for: \(string)")
        return
    }
    // Mark as self-generated
    down.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
    up.setIntegerValueField(.eventSourceUserData, value: kGoNhanhEventMarker)
    down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: utf16)
    down.post(tap: .cgSessionEventTap)
    up.post(tap: .cgSessionEventTap)
}

// MARK: - Notifications

extension Notification.Name {
    static let toggleVietnamese = Notification.Name("toggleVietnamese")
}

