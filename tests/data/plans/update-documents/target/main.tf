resource "terraform_data" "id" {}

resource "terraform_data" "foo-bar" {
  input = {
    json    = jsonencode({ replicas = 2, service = { ports = [80, 443] } })
    yaml    = yamlencode({ replicas = 2, enabled = true })
    unknown = jsonencode({ id = terraform_data.id.id })
    documents = [
      jsonencode({ name = "a", enabled = true }),
      "name: b\nenabled: true\n",
    ]
  }
}
