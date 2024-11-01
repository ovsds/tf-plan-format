group "default" {
  targets = ["bin", "image"]
}

target "base" {
  dockerfile = "Dockerfile"
  target = "base"
  contexts = {
    "base_build_image" = "docker-image://docker.io/library/rust:1.82.0-bookworm",
    "sources" = "../",
  }
}

target "bin-base" {
  inherits = ["base"]
  target = "bin"
  output = ["type=local,dest=bin/base"]
}

target "bin" {
  inherits = ["bin-base"]
  name = "bin-${item.name}"
  output = ["type=local,dest=bin/${item.name}"]
  args = {
    RUST_TARGET = "${item.rust_target}",
    RUST_LINKER = "${item.rust_linker}",
  }
  matrix = {
    item = [
      {
        name = "linux-amd64",
        rust_target="x86_64-unknown-linux-gnu",
        rust_linker="x86_64-linux-gnu-gcc",
      },
      {
        name = "linux-arm64",
        rust_target="aarch64-unknown-linux-gnu",
        rust_linker="aarch64-linux-gnu-gcc",
      },
      {
        name = "darwin-amd64",
        rust_target="x86_64-apple-darwin",
        rust_linker="rust-lld",
      },
      {
        name = "darwin-arm64",
        rust_target="aarch64-apple-darwin",
        rust_linker="rust-lld",
      },
    ]
  }
}

variable "IMAGE_TAG" {}

target "image" {
  inherits = ["base"]
  target = "image"
  output = ["type=registry"]
  tags = [IMAGE_TAG]
  platforms = ["linux/amd64", "linux/arm64"]
}

target "image-local" {
  inherits = ["image"]
  output = ["type=docker"]
  platforms = []
}
