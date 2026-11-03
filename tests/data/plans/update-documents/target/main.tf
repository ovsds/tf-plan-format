resource "terraform_data" "foo-bar" {
  input = {
    json = jsonencode({ replicas = 2, service = { ports = [80, 443] } })
    yaml = yamlencode({ replicas = 2, enabled = true })
    documents = [
      jsonencode({ name = "a", enabled = true }),
      "name: b\nenabled: true\n",
    ]
  }
}
