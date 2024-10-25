resource "null_resource" "foo-bar" {
  triggers = {
    always_run = "${timestamp()}"
  }
}
