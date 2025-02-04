# @moneko/raw-import

SWC 插件 - 将带 `?raw` 查询参数的导入转换为文件原始内容

[![npm version](https://img.shields.io/npm/v/@moneko/raw-import)](https://www.npmjs.com/package/@moneko/raw-import)

## 特性

- 📦 将文件内容作为字符串导入
- ⚡ SWC 原生速度
- 🛠️ 支持相对路径和 node_modules 解析

## 安装

```bash
npm install --save-dev @moneko/raw-import
# 或
yarn add -D @moneko/raw-import
# 或
pnpm add -D @moneko/raw-import
```

## 配置

在 `.swcrc` 中添加配置：

```js
// .swcrc

module.exports = {
  jsc: {
    experimental: {
      plugins: [
        [
          "@moneko/raw-import",
          {
            // 必须配置项目根目录（通常为 process.cwd()）
            rootDir: process.cwd(),
          },
        ],
      ],
    },
  },
};
```

## 使用示例

### 基本用法

```js
// 输入
import cssContent from "./styles.css?raw";

// 输出
const cssContent = "body { color: red; }";
```

### 从 node_modules 导入

```js
// 输入
import license from "some-pkg/LICENSE?raw";

// 输出
const license = "MIT License...";
```

## 注意事项

1. **路径处理**：

   - 确保路径中包含 `?raw` 查询参数

## 许可证

MIT
