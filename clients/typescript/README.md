## Generate JavaScript bundle

```sh
cd clients\typescript
deno run -A ./tools/bundle.ts
```

## Symlinks

```sh
cd clients\typescript
mklink /D javascript ..\..\libs\setu-codegen\clients\javascript
mklink /D src ..\..\libs\setu-codegen\clients\typescript
```
