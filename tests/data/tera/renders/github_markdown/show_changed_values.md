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
id: "70ab275d-63ef-e14a-1bd4-75d2a3a580f7"
input: "foo"
output: "foo"
triggers_replace: null
```

</details>
</details>
<details>
<summary>🟰tests/data/plans/no-op/terraform.tfplan.json</summary>
<details>
<summary>🟰terraform_data.foo-bar
</summary>

```
id: "0f61b5b9-e9e3-1625-f62b-501a232653f9"
input: "foo"
output: "foo"
triggers_replace: null
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
base64: sensitive -> sensitive
hex: sensitive -> sensitive
keepers: null
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
base64: sensitive -> sensitive
hex: sensitive -> sensitive
keepers: null
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
id: "69e12f7b-2747-94ba-2ead-275f4616c74b"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
</details>
<details>
<summary>🔄tests/data/plans/update/terraform.tfplan.json</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
id: "72285066-beaf-bd58-0c9f-0c5e7ae166a2"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
</details>
