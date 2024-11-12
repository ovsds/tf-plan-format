terraform {
  required_providers {
    random = {
      source = "hashicorp/random"
      version = "3.6.3"
    }
  }
}

resource "random_bytes" "test" {
  length = 4
}
