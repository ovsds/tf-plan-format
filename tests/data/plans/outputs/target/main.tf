resource "terraform_data" "foo-bar" {
  input = "bar"
}

output "same" {
  value = "same"
}

output "changed" {
  value = "after"
}

output "added" {
  value = "added"
}

output "secret" {
  value     = "secret-after"
  sensitive = true
}

output "unknown" {
  value = terraform_data.foo-bar.output
}
