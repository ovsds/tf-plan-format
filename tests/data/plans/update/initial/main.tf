terraform {
  required_providers {
    local = {
      source = "hashicorp/local"
      version = "2.5.2"
    }
  }
}

resource "terraform_data" "replacement" {
  input = "foo"
}
