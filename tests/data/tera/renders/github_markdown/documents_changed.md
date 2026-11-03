<!-- tf-plan-format -->
<details>
<summary>🔄 <b>opentofu.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
  id: "5de48861-0a5f-68d0-4c75-879478e15bb1"
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
-   yaml: | yaml
-     "enabled": true
-     "replicas": 1
+ output: (known after apply)
  triggers_replace: null
```

</details>
<details>
<summary>🔄 <b>terraform.tfplan.json</b> +0 ~1 −0</summary>

```diff
@@ 🔄 terraform_data.foo-bar @@
  id: "831d3664-a633-17f6-95a7-6bc28a6a0af2"
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
-   yaml: | yaml
-     "enabled": true
-     "replicas": 1
+ output: (known after apply)
  triggers_replace: null
```

</details>
