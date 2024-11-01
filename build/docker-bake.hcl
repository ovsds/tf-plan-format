target "base" {
  dockerfile = "Dockerfile"
  target = "base"
  contexts = {
    "base_build_image" = "docker-image://docker.io/library/rust:1.82.0-bookworm",
    "sources" = "../",
  }
}

target "bin" {
  inherits = ["base"]
  target = "bin"
  output = ["type=local,dest=bin/local"]
}

target "cross-bin" {
  inherits = ["bin"]
  name = "bin-${item.os}-${item.arch}"
  target = "cross_bin"
  output = ["type=local,dest=bin/${item.os}-${item.arch}"]
  matrix = {
    item = [
      {
        os = "linux",
        arch = "amd64",
        rust_target="x86_64-unknown-linux-gnu",
        rust_linker="x86_64-linux-gnu-gcc",
      },
      {
        os = "linux",
        arch = "arm64",
        rust_target="aarch64-unknown-linux-gnu",
        rust_linker="aarch64-linux-gnu-gcc",
      },
      {
        os = "darwin",
        arch = "amd64",
        rust_target="x86_64-apple-darwin",
        rust_linker="rust-lld",
      },
      {
        os = "darwin",
        arch = "arm64",
        rust_target="aarch64-apple-darwin",
        rust_linker="rust-lld",
      },
    ]
  }
  args = {
    RUST_TARGET = "${item.rust_target}",
    RUST_LINKER = "${item.rust_linker}",
  }
}

variable "IMAGE_TAG" {}

target "cross-image" {
  inherits = ["bin"]
  target = "cross_image"
  output = ["type=image"]
  contexts = {
    "binaries" = "./bin",
  }
  tags = [IMAGE_TAG]
  platforms = ["linux/amd64", "linux/arm64", "darwin/amd64", "darwin/arm64"]
}
