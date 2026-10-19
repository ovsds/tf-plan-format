resource "terraform_data" "foo-bar" {
  input = "bar"
}

# The config depends on an output unknown at plan time, so the read is deferred to apply and the missing state file is never opened
data "terraform_remote_state" "foo-bar" {
  backend = "local"
  config = {
    path = "missing.tfstate"
  }
  defaults = {
    value = terraform_data.foo-bar.output
  }
}
