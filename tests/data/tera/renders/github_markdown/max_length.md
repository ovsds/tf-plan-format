<!-- tf-plan-format -->
<details>
<summary>✅ <b>create/opentofu.tfplan.json</b> - Plan: 1 to add, 0 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>✅ <code>terraform_data.foo-bar</code>
</summary>

_Values not shown to fit the length limit_

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

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>delete-create/terraform.tfplan.json</b> - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ <code>null_resource.foo-bar</code>
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>❌ <b>delete/opentofu.tfplan.json</b> - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>❌ <code>terraform_data.foo-bar</code>
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>❌ <b>delete/terraform.tfplan.json</b> - Plan: 0 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>❌ <code>terraform_data.foo-bar</code>
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary><b>no-op/opentofu.tfplan.json</b> - No changes</summary>
<blockquote>
No resource changes
</blockquote>
</details>
<details>
<summary><b>no-op/terraform.tfplan.json</b> - No changes</summary>
<blockquote>
No resource changes
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

_Values not shown to fit the length limit_

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

_Values not shown to fit the length limit_

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

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>♻️ <b>sensitive/terraform.tfplan.json</b> - Plan: 1 to add, 0 to change, 1 to destroy</summary>
<blockquote>
<details>
<summary>♻️ <code>random_bytes.test</code>
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>update-array/opentofu.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>
<details>
<summary>🔄 <b>update-array/terraform.tfplan.json</b> - Plan: 0 to add, 1 to change, 0 to destroy</summary>
<blockquote>
<details>
<summary>🔄 <code>terraform_data.foo-bar</code>
</summary>

_Values not shown to fit the length limit_

</details>
</blockquote>
</details>

_Output limited to 5000 bytes: the last 2 plans, values of 9 changes and outputs of 2 plans not shown_
