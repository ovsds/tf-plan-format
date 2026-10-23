<!-- tf-plan-format -->
<details>
<summary>✅ <b>create/opentofu.tfplan.json</b> +1 ~0 −0</summary>

```diff
@@ ✅ terraform_data.foo-bar @@
+ id: (known after apply)
+ input: "foo"
+ output: (known after apply)
+ triggers_replace: null
```

</details>
<details>
<summary>✅ <b>create/terraform.tfplan.json</b> +1 ~0 −0</summary>

```diff
@@ ✅ terraform_data.foo-bar @@
+ id: (known after apply)
+ input: "foo"
+ output: (known after apply)
+ triggers_replace: null
```

</details>
<details>
<summary>♻️ <b>delete-create/opentofu.tfplan.json</b> +1 ~0 −1</summary>

```diff
@@ ♻️ null_resource.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>♻️ <b>delete-create/terraform.tfplan.json</b> +1 ~0 −1</summary>

```diff
@@ ♻️ null_resource.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>❌ <b>delete/opentofu.tfplan.json</b> +0 ~0 −1</summary>

```diff
@@ ❌ terraform_data.foo-bar @@
- id: "7ab2f2ee-c808-082c-93c7-d2a00826c3e4"
- input: "foo"
- output: "foo"
- triggers_replace: null
```

</details>
<details>
<summary>❌ <b>delete/terraform.tfplan.json</b> +0 ~0 −1</summary>

```diff
@@ ❌ terraform_data.foo-bar @@
- id: "96202d3f-5e6b-8c7f-8e5a-7d1599601bd8"
- input: "foo"
- output: "foo"
- triggers_replace: null
```

</details>
<details>
<summary>✅🔄❌ <b>outputs/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
! input: "foo" -> "bar"
! output: "foo" -> (known after apply)
@@ ✅🔄❌ Outputs @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>✅🔄❌ <b>outputs/terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
! input: "foo" -> "bar"
! output: "foo" -> (known after apply)
@@ ✅🔄❌ Outputs @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>🔄 <b>read/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
! input: "foo" -> "bar"
! output: "foo" -> (known after apply)
```

</details>
<details>
<summary>🔄 <b>read/terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
! input: "foo" -> "bar"
! output: "foo" -> (known after apply)
```

</details>
<details>
<summary>♻️ <b>sensitive/opentofu.tfplan.json</b> +1 ~0 −1</summary>

```diff
@@ ♻️ random_bytes.test @@
! base64: sensitive -> sensitive (known after apply)
! hex: sensitive -> sensitive (known after apply)
! length: 4 -> 8
```

</details>
<details>
<summary>♻️ <b>sensitive/terraform.tfplan.json</b> +1 ~0 −1</summary>

```diff
@@ ♻️ random_bytes.test @@
! base64: sensitive -> sensitive (known after apply)
! hex: sensitive -> sensitive (known after apply)
! length: 4 -> 8
```

</details>
<details>
<summary>🔄 <b>update-array/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>🔄 <b>update-array/terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>🔄 <b>update/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
! input: "foo" -> "bar"
! output: "foo" -> (known after apply)
```

</details>
<details>
<summary>🔄 <b>update/terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
! input: "foo" -> "bar"
! output: "foo" -> (known after apply)
```

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

_Output limited to 4000 bytes: values of 4 changes and outputs of 2 plans not shown_
