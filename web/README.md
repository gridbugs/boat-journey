# Template2023 Web

## Install
```
$ cargo install wasm-pack
$ npm install
```

## Run Development Server
```
$ NODE_OPTIONS=--openssl-legacy-provider npm run serve
```

## Build
Artifacts will be placed in "./dist":

### Development (faster to build)
```
$ NODE_OPTIONS=--openssl-legacy-provider npm run build
```

### Production (faster to execute)
```
$ NODE_OPTIONS=--openssl-legacy-provider npm run build-production
```
