class Cartograph < Formula
  desc "Ultra-Fast Native AST Code Graph Intelligence Engine & CLI"
  homepage "https://github.com/PankajRedekar/cartograph"
  url "https://github.com/PankajRedekar/cartograph/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA256"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/PankajRedekar/cartograph.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/cartograph-cli")
  end

  test do
    assert_match "cartograph", shell_output("#{bin}/cartograph --version")
    assert_match "Cartograph Code Graph CLI", shell_output("#{bin}/cartograph --help")
  end
end
