class QuantumnCode < Formula
  desc "AI-powered coding assistant CLI"
  homepage "https://github.com/Akatsuki2r/QuantumCode"
  version "0.1.0"
  license "MIT"
  head "https://github.com/Akatsuki2r/QuantumCode.git", branch: "main"

  on_macos do
    on_intel do
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_arm do
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_arm do
      url "https://github.com/Akatsuki2r/QuantumCode/releases/download/v0.1.0/quantumn-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "quantumn"
  end

  test do
    system "#{bin}/quantumn", "--version"
  end
end