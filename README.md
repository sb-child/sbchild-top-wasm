# sbchild.top WASM 组件

本项目从 wasm-pack 模板生成。记得安装 wasm-pack CLI 工具。

- tutorials: https://rustwasm.github.io/docs/wasm-pack/tutorials/index.html
- template-docs: https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/index.html

## 介绍

本项目用于 <https://sbchild.top/> 网站([GitHub Repo](https://github.com/sb-child/sbchild.top))。

- `src/lightbox`: 图床API，用于灯箱组件的 Worker 后端
- `src/lightbox_image`: 图像处理函数，用于前端灯箱组件

## 如何使用本项目

### 构建

Scope 必须是 `sb-child`，来确保发布到正确的 Registry。

```
wasm-pack build --scope sb-child
```

### 测试(现在没有写)

```
wasm-pack test --headless --firefox
```

### 发布到 GitHub NPM Registry

在 `~/.npmrc` 添加这两行，把 `TOKEN` 替换为你的 [personal access token](https://github.com/settings/tokens/new):

```sh
@sb-child:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=TOKEN
```

Token 必须用 Classic 类型，需要启用 `read:packages`,`write:packages` Scope。

然后执行 `wasm-pack publish`。

### 安装到你的项目

## License

MIT license.
