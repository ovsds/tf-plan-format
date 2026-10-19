resource "terraform_data" "foo-bar" {
  input = "foo"
}

output "same" {
  value = "same"
}

output "changed" {
  value = "before"
}

output "removed" {
  value = "removed"
}

output "secret" {
  value     = "secret-before"
  sensitive = true
}

output "unknown" {
  value = terraform_data.foo-bar.output
}
