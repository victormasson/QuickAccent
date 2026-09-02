class Quickaccent < Formula
  desc "Cross-platform accent character picker (macOS menu bar)"
  homepage "https://github.com/victormasson/QuickAccent"
  license "MIT"

  # Prebuilt universal binary (no Rust): dist/macos/install.sh
  # https://github.com/victormasson/QuickAccent/releases/tag/continuous

  head "https://github.com/victormasson/QuickAccent.git", branch: "master"

  # Local source build (no network)
  # head "file:///Users/aclydes/Coding/QuickAccent", using: :git

  depends_on "rust" => :build
  depends_on :macos

  def install
    system "cargo", "build", "--release"

    # Install as macOS .app bundle (LSUIElement=true → no Dock icon)
    app = prefix/"QuickAccent.app"
    (app/"Contents/MacOS").mkpath
    (app/"Contents/Resources").mkpath
    cp "target/release/quickaccent", app/"Contents/MacOS/quickaccent"
    cp "dist/macos/AppIcon.icns", app/"Contents/Resources/AppIcon.icns"

    # Write minimal Info.plist (agent app, no Dock icon)
    (app/"Contents/Info.plist").write <<~PLIST
      <?xml version="1.0" encoding="UTF-8"?>
      <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
      <plist version="1.0">
      <dict>
        <key>CFBundleExecutable</key>
        <string>quickaccent</string>
        <key>CFBundleIdentifier</key>
        <string>com.quickaccent.app</string>
        <key>CFBundleName</key>
        <string>QuickAccent</string>
        <key>CFBundleIconFile</key>
        <string>AppIcon</string>
        <key>CFBundlePackageType</key>
        <string>APPL</string>
        <key>CFBundleVersion</key>
        <string>1.1.1</string>
        <key>CFBundleShortVersionString</key>
        <string>1.1.1</string>
        <key>LSUIElement</key>
        <true/>
        <key>LSMinimumSystemVersion</key>
        <string>11.0</string>
      </dict>
      </plist>
    PLIST
  end

  service do
    run [opt_prefix/"QuickAccent.app/Contents/MacOS/quickaccent"]
    keep_alive true
    run_at_load true
    log_path "/tmp/quickaccent.log"
    error_log_path "/tmp/quickaccent.err"
  end

  def post_install
    # Symlink LaunchAgent into user's LaunchAgents directory
    launch_agents = Pathname.new(ENV["HOME"]) + "Library/LaunchAgents"
    launch_agents.mkpath
    (launch_agents/"com.quickaccent.app.plist").make_symlink(prefix/"quickaccent.plist")
  end

  def caveats
    <<~EOS
      QuickAccent is now installed and will launch at login via LaunchAgent.

      IMPORTANT: Grant Accessibility permission:
        System Settings → Privacy & Security → Accessibility → add QuickAccent (or your terminal).

      Prebuilt (no Rust) alternative:
        curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/macos/install.sh | bash

      To start immediately (without logging out/in):
        launchctl load ~/Library/LaunchAgents/com.quickaccent.app.plist

      To stop:
        launchctl unload ~/Library/LaunchAgents/com.quickaccent.app.plist

      Logs: /tmp/quickaccent.log and /tmp/quickaccent.err
    EOS
  end

  test do
    assert_match "QuickAccent", shell_output("#{bin}/quickaccent --help 2>&1 || true")
  end
end
