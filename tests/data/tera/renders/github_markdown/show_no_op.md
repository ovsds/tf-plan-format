<!-- tf-plan-format -->
<details>
<summary>✅ <b>create/opentofu.tfplan.json</b> - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>✅ <code>terraform_data.foo-bar</code>
</summary>

```
id: (known after apply)
input: "foo"
output: (known after apply)
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>✅ <b>create/terraform.tfplan.json</b> - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>✅ <code>terraform_data.foo-bar</code>
</summary>

```
id: (known after apply)
input: "foo"
output: (known after apply)
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>delete-create/opentofu.tfplan.json</b> - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ <code>null_resource.foo-bar</code>
</summary>

```
id: "8077357928023619263" -> (known after apply)
triggers:
  always_run: "2026-10-18T19:59:22Z" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>delete-create/terraform.tfplan.json</b> - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ <code>null_resource.foo-bar</code>
</summary>

```
id: "4525788878524015586" -> (known after apply)
triggers:
  always_run: "2024-10-25T21:40:19Z" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>❌ <b>delete/opentofu.tfplan.json</b> - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>❌ <code>terraform_data.foo-bar</code>
</summary>

```
id: "7ab2f2ee-c808-082c-93c7-d2a00826c3e4"
input: "foo"
output: "foo"
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>❌ <b>delete/terraform.tfplan.json</b> - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>❌ <code>terraform_data.foo-bar</code>
</summary>

```
id: "96202d3f-5e6b-8c7f-8e5a-7d1599601bd8"
input: "foo"
output: "foo"
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>💤 <b>no-op/opentofu.tfplan.json</b> - Plan: 0 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>💤 <code>terraform_data.foo-bar</code>
</summary>

_No value changes_

</details>
</blockquote>
</details>
<details>
<summary>💤 <b>no-op/terraform.tfplan.json</b> - Plan: 0 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>💤 <code>terraform_data.foo-bar</code>
</summary>

_No value changes_

</details>
</blockquote>
</details>
<details>
<summary><b>no-resources/opentofu.tfplan.json</b> - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary><b>no-resources/terraform.tfplan.json</b> - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary>✅🔄❌ <b>outputs/opentofu.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌ Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
secret: sensitive -> sensitive (changed)
unknown: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>✅🔄❌ <b>outputs/terraform.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌ Outputs
</summary>

```
added: null -> "added"
changed: "before" -> "after"
removed: "removed" -> null
secret: sensitive -> sensitive (changed)
unknown: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>read/opentofu.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>read/terraform.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>sensitive/opentofu.tfplan.json</b> - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ <code>random_bytes.test</code>
</summary>

```
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
length: 4 -> 8
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>sensitive/terraform.tfplan.json</b> - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ <code>random_bytes.test</code>
</summary>

```
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
length: 4 -> 8
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>update-array/opentofu.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
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
</blockquote>
</details>
<details>
<summary>🔄 <b>update-array/terraform.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
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
</blockquote>
</details>
<details>
<summary>🔄 <b>update/opentofu.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>update/terraform.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
