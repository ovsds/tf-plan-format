<!-- tf-plan-format -->
<details>
<summary>🔄 <b>opentofu.tfplan.json</b> +0 ~1 −0</summary>

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
-   yaml: | yaml
-     "enabled": true
-     "replicas": 1
+ output: (known after apply)
```

</details>
<details>
<summary>🔄 <b>terraform.tfplan.json</b> +0 ~1 −0</summary>

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
-   yaml: | yaml
-     "enabled": true
-     "replicas": 1
+ output: (known after apply)
```

</details>
