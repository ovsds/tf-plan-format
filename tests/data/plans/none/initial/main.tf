terraform {
  required_providers {
    local = {
      source = "hashicorp/local"
      version = "2.5.2"
    }
  }
}

data "local_file" "foo" {
  filename = "${path.module}/../foo"
}
