<!-- tf-plan-format -->
<details>
<summary>✅🔄 <b>opentofu.tfplan.json</b> +1 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
  id: "6246d726-a03f-96e4-9078-2c4a25f0e580"
  input:
    documents:
      - | json
        {
-         "enabled": false,
+         "enabled": true,
          "name": "a"
        }
      - | yaml
        name: b
-       enabled: false
+       enabled: true
    json: | json
      {
-       "replicas": 1,
+       "replicas": 2,
        "service": {
          "ports": [
-           80
+           80,
+           443
          ]
        }
      }
-   unknown: | json
-     {
-       "id": "a"
-     }
+   unknown: (known after apply)
    yaml: | yaml
      "enabled": true
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
  triggers_replace: null
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
  id: "e1c31aaa-d241-9bbf-d3d7-50a9b095e7d2"
  input:
    documents:
      - | json
        {
-         "enabled": false,
+         "enabled": true,
          "name": "a"
        }
      - | yaml
        name: b
-       enabled: false
+       enabled: true
    json: | json
      {
-       "replicas": 1,
+       "replicas": 2,
        "service": {
          "ports": [
-           80
+           80,
+           443
          ]
        }
      }
-   unknown: | json
-     {
-       "id": "a"
-     }
+   unknown: (known after apply)
    yaml: | yaml
      "enabled": true
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
  triggers_replace: null
@@ ✅ terraform_data.id @@
+ id: (known after apply)
+ input: null
+ output: null
+ triggers_replace: null
```

</details>
