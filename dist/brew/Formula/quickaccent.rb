class Quickaccent < Formula
  desc "Cross-platform accent character picker (macOS menu bar)"
  homepage "https://github.com/victormasson/QuickAccent"
  license "MIT"

  # head "https://github.com/victormasson/QuickAccent.git", branch: "master"

  # Local source build (no network)
  head "file:///Users/aclydes/Coding/QuickAccent", using: :git

  depends_on "rust" => :build
  depends_on :macos

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/quickaccent"
  end

  service do
    run [opt_bin/"quickaccent"]
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
