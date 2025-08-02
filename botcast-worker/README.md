# botcast-worker

## OpenAPI Generator

```bash
openapi-generator-cli generate -i spec.yml -g rust-axum -o ./crates/openapi_gen
```

```bash
openapi-generator-cli generate -i spec.yml -g rust -o ./crates/openapi_client --additional-properties=packageName=openapi_client
```
