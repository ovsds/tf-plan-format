resource "terraform_data" "foo-bar" {
  input = {
    json    = jsonencode({ replicas = 1, service = { ports = [80] } })
    yaml    = yamlencode({ replicas = 1, enabled = true })
    unknown = jsonencode({ id = "a" })
    documents = [
      jsonencode({ name = "a", enabled = false }),
      "name: b\nenabled: false\n",
    ]
  }
}
