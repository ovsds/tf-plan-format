<details>
<summary>✅create/opentofu.tfplan.json - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<details>
<summary>✅terraform_data.foo-bar
</summary>

```
id: (known after apply)
input: "foo"
output: (known after apply)
triggers_replace: null
```

</details>
</details>
<details>
<summary>✅create/terraform.tfplan.json - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<details>
<summary>✅terraform_data.foo-bar
</summary>

```
id: (known after apply)
input: "foo"
output: (known after apply)
triggers_replace: null
```

</details>
</details>
<details>
<summary>♻️delete-create/opentofu.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<details>
<summary>♻️null_resource.foo-bar
</summary>

```
id: "8077357928023619263" -> (known after apply)
triggers:
  always_run: "2026-10-18T19:59:22Z" -> (known after apply)
```

</details>
</details>
<details>
<summary>♻️delete-create/terraform.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<details>
<summary>♻️null_resource.foo-bar
</summary>

```
id: "4525788878524015586" -> (known after apply)
triggers:
  always_run: "2024-10-25T21:40:19Z" -> (known after apply)
```

</details>
</details>
<details>
<summary>❌delete/opentofu.tfplan.json - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<details>
<summary>❌terraform_data.foo-bar
</summary>

```
id: "7ab2f2ee-c808-082c-93c7-d2a00826c3e4"
input: "foo"
output: "foo"
triggers_replace: null
```

</details>
</details>
<details>
<summary>❌delete/terraform.tfplan.json - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<details>
<summary>❌terraform_data.foo-bar
</summary>

```
id: "96202d3f-5e6b-8c7f-8e5a-7d1599601bd8"
input: "foo"
output: "foo"
triggers_replace: null
```

</details>
</details>
<details>
<summary>🟰no-op/opentofu.tfplan.json - No changes</summary>
<details>
<summary>🟰terraform_data.foo-bar
</summary>

```

```

</details>
</details>
<details>
<summary>🟰no-op/terraform.tfplan.json - No changes</summary>
<details>
<summary>🟰terraform_data.foo-bar
</summary>

```

```

</details>
</details>
<details>
<summary>no-resources/opentofu.tfplan.json - No changes</summary>
No resource changes
</details>
<details>
<summary>no-resources/terraform.tfplan.json - No changes</summary>
No resource changes
</details>
<details>
<summary>🔄outputs/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
unknown: "foo" -> (known after apply)
```

</details>
</details>
<details>
<summary>🔄outputs/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
unknown: "foo" -> (known after apply)
```

</details>
</details>
<details>
<summary>♻️sensitive/opentofu.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<details>
<summary>♻️random_bytes.test
</summary>

```
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
length: 4 -> 8
```

</details>
</details>
<details>
<summary>♻️sensitive/terraform.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<details>
<summary>♻️random_bytes.test
</summary>

```
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
length: 4 -> 8
```

</details>
</details>
<details>
<summary>🔄update-array/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input:
  names:
    - ...
    - null -> "x"
    - ...
    - "c" -> null
  ports:
    - port: 80 -> 443
    - ...
output: {names: ["a", "b", "c"], ports: [{name: "http", port: 80}, {name: "ssh", port: 22}]} -> (known after apply)
```

</details>
</details>
<details>
<summary>🔄update-array/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input:
  names:
    - ...
    - null -> "x"
    - ...
    - "c" -> null
  ports:
    - port: 80 -> 443
    - ...
output: {names: ["a", "b", "c"], ports: [{name: "http", port: 80}, {name: "ssh", port: 22}]} -> (known after apply)
```

</details>
</details>
<details>
<summary>🔄update/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</details>
<details>
<summary>🔄update/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</details>
