resource "terraform_data" "foo-bar" {
  input = {
    names = ["a", "x", "b"]
    ports = [{ name = "http", port = 443 }, { name = "ssh", port = 22 }]
  }
}
