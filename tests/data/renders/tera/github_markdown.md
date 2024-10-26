<details>
<summary>create</summary>
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
<summary>delete</summary>
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
<summary>delete-create</summary>
<details>
<summary>♻️null_resource.foo-bar
</summary>

```
id: "4525788878524015586" -> null
triggers: {"always_run":"2024-10-25T21:40:19Z"} -> {}
```

</details>
</details>
<details>
<summary>no-op</summary>
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
<summary>no-resources</summary>
No resource changes
</details>
<details>
<summary>update</summary>
<details>
<summary>🔄terraform_data.foo-bar
</summary>

```
id: "72285066-beaf-bd58-0c9f-0c5e7ae166a2"
input: "foo" -> "bar"
output: "foo" -> null
triggers_replace: null
```

</details>
</details>
