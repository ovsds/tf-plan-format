<!-- tf-plan-format -->
<details>
<summary>✅ <b>create/opentofu.tfplan.json</b> +1 ~0 −0</summary>
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
<summary>✅ <b>create/terraform.tfplan.json</b> +1 ~0 −0</summary>
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
<summary>♻️ <b>delete-create/opentofu.tfplan.json</b> +1 ~0 −1</summary>
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
<summary>♻️ <b>delete-create/terraform.tfplan.json</b> +1 ~0 −1</summary>
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
<summary>❌ <b>delete/opentofu.tfplan.json</b> +0 ~0 −1</summary>
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
<summary>❌ <b>delete/terraform.tfplan.json</b> +0 ~0 −1</summary>
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
<summary>✅🔄❌ <b>outputs/opentofu.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
id: "567018a0-5e04-1bae-7c35-1b64599f8bb3"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
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
<summary>✅🔄❌ <b>outputs/terraform.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
id: "a5b815d4-2349-e64a-9c56-ad4f7abee3d2"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
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
<summary>🔄 <b>read/opentofu.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
id: "f7a1ff7f-088b-ca29-6456-8a47bd2ce523"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>read/terraform.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
id: "2216c960-d6d5-6a9e-ea6c-1df838857438"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>sensitive/opentofu.tfplan.json</b> +1 ~0 −1</summary>
<blockquote>
<details>
<summary>♻️ <code>random_bytes.test</code>
</summary>

```
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
keepers: null
length: 4 -> 8
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>sensitive/terraform.tfplan.json</b> +1 ~0 −1</summary>
<blockquote>
<details>
<summary>♻️ <code>random_bytes.test</code>
</summary>

```
base64: sensitive -> sensitive (known after apply)
hex: sensitive -> sensitive (known after apply)
keepers: null
length: 4 -> 8
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>update-array/opentofu.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
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
</blockquote>
</details>
<details>
<summary>🔄 <b>update-array/terraform.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
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
</blockquote>
</details>
<details>
<summary>🔄 <b>update/opentofu.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
id: "69e12f7b-2747-94ba-2ead-275f4616c74b"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>update/terraform.tfplan.json</b> +0 ~1 −0</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

```
id: "72285066-beaf-bd58-0c9f-0c5e7ae166a2"
input: "foo" -> "bar"
output: "foo" -> (known after apply)
triggers_replace: null
```

</details>
</blockquote>
</details>
<details>
<summary>💤 No changes (4)</summary>
<blockquote>
<div>💤 <b>no-op/opentofu.tfplan.json</b></div>
<div>💤 <b>no-op/terraform.tfplan.json</b></div>
<div>💤 <b>no-resources/opentofu.tfplan.json</b></div>
<div>💤 <b>no-resources/terraform.tfplan.json</b></div>
</blockquote>
</details>
