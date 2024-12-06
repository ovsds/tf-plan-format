<details>
<summary>✅tests/data/plans/create/terraform.tfplan.json</summary>
<details>
<summary>✅terraform_data.foo-bar
</summary>

```
input: "foo"
triggers_replace: null
```

</details>
</details>
<details>
<summary>♻️tests/data/plans/delete-create/terraform.tfplan.json</summary>
<details>
<summary>♻️null_resource.foo-bar
</summary>

```
id: "4525788878524015586" -> null
triggers:
  always_run: "2024-10-25T21:40:19Z" -> null
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
<summary>🟰tests/data/plans/no-op/terraform.tfplan.json</summary>
<details>
<summary>🟰terraform_data.foo-bar
</summary>

```

```

</details>
</details>
<details>
<summary>tests/data/plans/no-resources/terraform.tfplan.json</summary>
No resource changes
</details>
<details>
<summary>♻️tests/data/plans/sensitive/terraform.tfplan.json</summary>
<details>
<summary>♻️random_bytes.test
</summary>

```
base64: sensitive -> null
hex: sensitive -> null
length: 4 -> 8
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
output: "foo" -> null
```

</details>
</details>
