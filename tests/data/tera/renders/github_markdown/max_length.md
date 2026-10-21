<!-- tf-plan-format -->
<details>
<summary>✅ create/opentofu.tfplan.json - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>✅ terraform_data.foo-bar
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
<summary>✅ create/terraform.tfplan.json - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>✅ terraform_data.foo-bar
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
<summary>♻️ delete-create/opentofu.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ null_resource.foo-bar
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>♻️ delete-create/terraform.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ null_resource.foo-bar
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>❌ delete/opentofu.tfplan.json - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>❌ terraform_data.foo-bar
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>❌ delete/terraform.tfplan.json - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>❌ terraform_data.foo-bar
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>no-op/opentofu.tfplan.json - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary>no-op/terraform.tfplan.json - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary>no-resources/opentofu.tfplan.json - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary>no-resources/terraform.tfplan.json - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary>✅🔄❌ outputs/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌ Outputs
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>✅🔄❌ outputs/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
<details>
<summary>✅🔄❌ Outputs
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>🔄 read/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>🔄 read/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>
<details>
<summary>♻️ sensitive/opentofu.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ random_bytes.test
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>♻️ sensitive/terraform.tfplan.json - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ random_bytes.test
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>🔄 update-array/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>🔄 update-array/terraform.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>🔄 update/opentofu.tfplan.json - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 terraform_data.foo-bar
</summary>

```
input: "foo" -> "bar"
output: "foo" -> (known after apply)
```

</details>
</blockquote>
</details>

_Output limited to 5000 bytes: the last plan, values of 8 changes and outputs of 2 plans not shown_
