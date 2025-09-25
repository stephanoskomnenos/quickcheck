# QuickCheck Runner Node.js

Node.js 版本的 QuickCheck 测试运行器，使用 bun 作为包管理器，TypeScript 编写。

## 功能特性

- 基于 gRPC 的多语言测试框架
- 与 Rust 版本的 quickcheck_rpc 兼容
- 支持异步测试执行
- 错误处理和 panic 捕获
- 易于扩展的测试函数接口

## 安装依赖

```bash
cd quickcheck_runner_nodejs
bun install
```

## 快速开始

### 1. 创建测试函数

```typescript
import { TestFunction } from './src/server.js';

// 简单的加法测试
const addTest: TestFunction = {
  propertyName: 'property_add',
  execute(args: { a: number; b: number }) {
    return args.a + args.b;
  }
};

// 反转数组测试
const reverseTest: TestFunction = {
  propertyName: 'property_reverse',
  execute(args: { xs: number[] }) {
    return args.xs.slice().reverse();
  }
};
```

### 2. 启动测试服务器

```typescript
import { startServer } from './src/server.js';

// 启动加法测试服务器
const server = await startServer(addTest, '[::1]:50051');
console.log('Server is running...');
```

### 3. 运行服务器

```bash
# 开发模式（监听文件变化）
bun run dev

# 生产模式
bun run start

# 构建 TypeScript
bun run build
```

## API 参考

### TestFunction 接口

```typescript
interface TestFunction {
  propertyName: string;
  execute(args: any): any;
}
```

### startServer 函数

```typescript
function startServer(
  testFunction: TestFunction, 
  address?: string
): Promise<grpc.Server>
```

## 与 Rust 框架集成

Node.js runner 可以与 Rust 的 quickcheck 框架无缝集成：

### Rust 测试定义

```rust
use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, Property};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddArgs {
    a: i32,
    b: i32,
}

struct AddTest {
    endpoint: String,
}

impl Property for AddTest {
    type Args = AddArgs;
    type Return = i32;
    const PROPERTY_NAME: &'static str = "property_add";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() {
    let prop = AddTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    quickcheck(prop).await;
}
```

### 运行流程

1. Rust 测试框架生成测试数据
2. 通过 gRPC 调用 Node.js runner
3. Node.js 执行测试函数并返回结果
4. Rust 框架验证测试结果

## 示例测试函数

项目包含以下示例测试函数：

- **加法测试** (`property_add`) - 基本的算术运算
- **反转测试** (`property_reverse`) - 数组反转操作

## 开发指南

### 添加新的测试函数

1. 在 `src/server.ts` 中添加新的测试函数定义
2. 或者创建独立的测试模块：

```typescript
// src/tests/math.ts
export const multiplyTest: TestFunction = {
  propertyName: 'property_multiply',
  execute(args: { a: number; b: number }) {
    return args.a * args.b;
  }
};
```

### 错误处理

测试函数可以抛出错误来表示测试失败：

```typescript
const divisionTest: TestFunction = {
  propertyName: 'property_divide',
  execute(args: { a: number; b: number }) {
    if (args.b === 0) {
      throw new Error('Division by zero');
    }
    return args.a / args.b;
  }
};
```

### 异步测试

支持异步操作：

```typescript
const asyncTest: TestFunction = {
  propertyName: 'property_async',
  async execute(args: { delay: number }) {
    await new Promise(resolve => setTimeout(resolve, args.delay));
    return 'done';
  }
};
```

## 配置选项

### 服务器地址

默认地址为 `[::1]:50051`，可以通过参数自定义：

```typescript
// 使用自定义地址
await startServer(testFunction, 'localhost:8080');
```

### gRPC 选项

可以通过修改 `protoLoader.loadSync` 的选项来自定义 gRPC 行为。

## 故障排除

### 常见问题

1. **端口占用**: 确保 50051 端口未被其他进程占用
2. **依赖问题**: 运行 `bun install` 确保所有依赖正确安装
3. **TypeScript 错误**: 运行 `bun run build` 检查类型错误

### 调试模式

启用详细日志：

```typescript
// 在服务器启动前设置
process.env.DEBUG = 'grpc';
```

## 许可证

与主项目相同的许可证。
