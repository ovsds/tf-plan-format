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
<summary>no-op/opentofu.tfplan.json - No changes</summary>
No resource changes
</details>
<details>
<summary>no-op/terraform.tfplan.json - No changes</summary>
No resource changes
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
id: "567018a0-5e04-1bae-7c35-1b64599f8bb3"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
<details>
<summary>✅🔄❌Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
secret: sensitive
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
id: "a5b815d4-2349-e64a-9c56-ad4f7abee3d2"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
<details>
<summary>✅🔄❌Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
secret: sensitive
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
keepers: null
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
keepers: null
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
id: "6644bda8-d272-93dc-c92d-07c84994e980"
input:
  names:
    - "a"
    - null -> "x"
    - "b"
    - "c" -> null
  ports:
    - name: "http"
      port: 80 -> 443
    - name: "ssh"
      port: 22
output: {names: ["a", "b", "c"], ports: [{name: "http", port: 80}, {name: "ssh", port: 22}]} -> (known after apply)
triggers_replace: null
```

</details>
</details>
<details>
<summary>🔄update-array/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
id: "45e28af4-dcf3-e0d7-5b03-236af59f991b"
input:
  names:
    - "a"
    - null -> "x"
    - "b"
    - "c" -> null
  ports:
    - name: "http"
      port: 80 -> 443
    - name: "ssh"
      port: 22
output: {names: ["a", "b", "c"], ports: [{name: "http", port: 80}, {name: "ssh", port: 22}]} -> (known after apply)
triggers_replace: null
```

</details>
</details>
<details>
<summary>🔄update/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
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
<summary>🔄update/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
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
