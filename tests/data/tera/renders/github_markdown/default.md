<details>
<summary>✅tests/data/plans/create/opentofu.tfplan.json</summary>
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
<summary>✅tests/data/plans/create/terraform.tfplan.json</summary>
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
<summary>♻️tests/data/plans/delete-create/opentofu.tfplan.json</summary>
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
<summary>♻️tests/data/plans/delete-create/terraform.tfplan.json</summary>
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
<summary>❌tests/data/plans/delete/opentofu.tfplan.json</summary>
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
<summary>❌tests/data/plans/delete/terraform.tfplan.json</summary>
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
<summary>🟰tests/data/plans/no-op/opentofu.tfplan.json</summary>
<details>
<summary>🟰terraform_data.foo-bar
</summary>

```

```

</details>
</details>
<details>
<summary>🟰tests/data/plans/no-op/terraform.tfplan.json</summary>
<details>
<summary>🟰terraform_data.foo-bar
</summary>

```

```

</details>
</details>
<details>
<summary>tests/data/plans/no-resources/opentofu.tfplan.json</summary>
No resource changes
</details>
<details>
<summary>tests/data/plans/no-resources/terraform.tfplan.json</summary>
No resource changes
</details>
<details>
<summary>♻️tests/data/plans/sensitive/opentofu.tfplan.json</summary>
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
<summary>♻️tests/data/plans/sensitive/terraform.tfplan.json</summary>
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
<summary>🔄tests/data/plans/update/opentofu.tfplan.json</summary>
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
<summary>🔄tests/data/plans/update/terraform.tfplan.json</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</details>
