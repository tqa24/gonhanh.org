import SwiftUI

@main
struct GoNhanhApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var menuBar: MenuBarController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Chỉ ẩn dock icon nếu đã hoàn thành onboarding
        let hasCompleted = UserDefaults.standard.bool(forKey: SettingsKey.hasCompletedOnboarding)
        if hasCompleted {
            NSApp.setActivationPolicy(.accessory)
        }

        menuBar = MenuBarController()
    }

    func applicationWillTerminate(_ notification: Notification) {
        KeyboardHookManager.shared.stop()
    }
}
