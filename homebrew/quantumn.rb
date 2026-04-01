class Quantumn < Formula
  desc "AI-powered coding assistant CLI"
  homepage "https://github.com/Akatsuki2r/QuantumCode"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "quantumn"
  end

  test do
    assert_match "Quantumn Code", shell_output("#{bin}/quantumn --version")
  end
end