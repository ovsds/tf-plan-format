resource "terraform_data" "foo-bar" {
  input = {
    names = ["a", "b", "c"]
    ports = [{ name = "http", port = 80 }, { name = "ssh", port = 22 }]
  }
}
