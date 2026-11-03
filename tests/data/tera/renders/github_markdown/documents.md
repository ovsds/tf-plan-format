<!-- tf-plan-format -->
<details>
<summary>✅🔄 <b>opentofu.tfplan.json</b> +1 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
  input:
    documents:
      - | json
        ...
-         "enabled": false,
+         "enabled": true,
        ...
      - | yaml
        ...
-       enabled: false
+       enabled: true
    json: | json
      ...
-       "replicas": 1,
+       "replicas": 2,
      ...
-           80
+           80,
+           443
      ...
-   unknown: | json
-     {
-       "id": "a"
-     }
+   unknown: (known after apply)
    yaml: | yaml
      ...
-     "replicas": 1
+     "replicas": 2
- output:
-   documents:
-     - | json
-       {
-         "enabled": false,
-         "name": "a"
-       }
-     - | yaml
-       name: b
-       enabled: false
-   json: | json
-     {
-       "replicas": 1,
-       "service": {
-         "ports": [
-           80
-         ]
-       }
-     }
-   unknown: | json
-     {
-       "id": "a"
-     }
-   yaml: | yaml
-     "enabled": true
-     "replicas": 1
+ output: (known after apply)
@@ ✅ terraform_data.id @@
+ id: (known after apply)
+ input: null
+ output: null
+ triggers_replace: null
```

</details>
<details>
<summary>✅🔄 <b>terraform.tfplan.json</b> +1 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
  input:
    documents:
      - | json
        ...
-         "enabled": false,
+         "enabled": true,
        ...
      - | yaml
        ...
-       enabled: false
+       enabled: true
    json: | json
      ...
-       "replicas": 1,
+       "replicas": 2,
      ...
-           80
+           80,
+           443
      ...
-   unknown: | json
-     {
-       "id": "a"
-     }
+   unknown: (known after apply)
    yaml: | yaml
      ...
-     "replicas": 1
+     "replicas": 2
- output:
-   documents:
-     - | json
-       {
-         "enabled": false,
-         "name": "a"
-       }
-     - | yaml
-       name: b
-       enabled: false
-   json: | json
-     {
-       "replicas": 1,
-       "service": {
-         "ports": [
-           80
-         ]
-       }
-     }
-   unknown: | json
-     {
-       "id": "a"
-     }
-   yaml: | yaml
-     "enabled": true
-     "replicas": 1
+ output: (known after apply)
@@ ✅ terraform_data.id @@
+ id: (known after apply)
+ input: null
+ output: null
+ triggers_replace: null
```

</details>
