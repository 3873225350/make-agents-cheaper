class MakeAgentsCheaper < Formula
  desc "Audit and evaluate prompt-cache friendliness in coding-agent workflows"
  homepage "https://github.com/3873225350/make-agents-cheaper"
  url "https://github.com/3873225350/make-agents-cheaper/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
  end

  test do
    system "#{bin}/make-agents-cheaper", "--help"
  end
end
