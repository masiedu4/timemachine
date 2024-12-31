class Timemachine < Formula
  desc "A powerful file versioning tool for tracking and managing file changes"
  homepage "https://github.com/masiedu4/timemachine"
  version "0.1.0"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/masiedu4/timemachine/releases/download/v#{version}/timemachine-macos-arm64.tar.gz"
      # sha256 will be added after first release
    else
      url "https://github.com/masiedu4/timemachine/releases/download/v#{version}/timemachine-macos-amd64.tar.gz"
      # sha256 will be added after first release
    end
  end

  def install
    bin.install "timemachine"
    
    # Install shell completions
    bash_completion.install "completions/timemachine.bash" => "timemachine"
    zsh_completion.install "completions/_timemachine" => "_timemachine"
    fish_completion.install "completions/timemachine.fish"
  end

  test do
    system "#{bin}/timemachine", "--version"
  end
end
