<!-- tf-plan-format -->
<details>
<summary>✅ <b>create/opentofu.tfplan.json</b> +1 ~0 −0</summary>

```diff
@@ ✅ terraform_data.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>✅ <b>create/terraform.tfplan.json</b> +1 ~0 −0</summary>

```diff
@@ ✅ terraform_data.foo-bar @@
 Values not shown to fit the length limit
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
 Values not shown to fit the length limit
```

</details>
<details>
<summary>❌ <b>delete/terraform.tfplan.json</b> +0 ~0 −1</summary>

```diff
@@ ❌ terraform_data.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>✅🔄❌ <b>outputs/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
@@ ✅🔄❌ Outputs @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>✅🔄❌ <b>outputs/terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
@@ ✅🔄❌ Outputs @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>🔄 <b>read/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>🔄 <b>read/terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>♻️ <b>sensitive/opentofu.tfplan.json</b> +1 ~0 −1</summary>

```diff
@@ ♻️ random_bytes.test @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>♻️ <b>sensitive/terraform.tfplan.json</b> +1 ~0 −1</summary>

```diff
@@ ♻️ random_bytes.test @@
 Values not shown to fit the length limit
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
<summary>✅🔄 <b>update-documents/opentofu.tfplan.json</b> +1 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
@@ ✅ terraform_data.id @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>✅🔄 <b>update-documents/terraform.tfplan.json</b> +1 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
@@ ✅ terraform_data.id @@
 Values not shown to fit the length limit
```

</details>
<details>
<summary>🔄 <b>update/opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
 Values not shown to fit the length limit
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
<summary>💤 No changes (3)</summary>
<blockquote>
<div>💤 <b>no-op/opentofu.tfplan.json</b></div>
<div>💤 <b>no-op/terraform.tfplan.json</b></div>
<div>💤 <b>no-resources/opentofu.tfplan.json</b></div>
</blockquote>
</details>

_Output limited to 4000 bytes: the last plan, values of 19 changes and outputs of 2 plans not shown_
