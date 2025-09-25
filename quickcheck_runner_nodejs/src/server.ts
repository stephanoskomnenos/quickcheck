import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 加载 proto 文件
const PROTO_PATH = path.join(__dirname, '../proto/pbt_service.proto');
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});

const pbtProto = grpc.loadPackageDefinition(packageDefinition) as any;

// 测试状态枚举
enum TestStatus {
  PASSED = 0,
  FAILED = 1,
  INVALID_INPUT = 2,
}

// 测试函数接口
interface TestFunction {
  propertyName: string;
  execute(args: any): any;
}

// 创建测试函数服务器
function createTestServer(testFunction: TestFunction) {
  const server = new grpc.Server();
  
  server.addService(pbtProto.pbt_service.TestRunner.service, {
    execute: (call: any, callback: any) => {
      const request = call.request;
      
      // 验证属性名称
      if (request.property_name !== testFunction.propertyName) {
        callback({
          code: grpc.status.NOT_FOUND,
          message: `Property '${request.property_name}' not found. This runner only supports '${testFunction.propertyName}'`
        });
        return;
      }
      
      try {
        // 解析 JSON 参数
        const args = JSON.parse(request.test_data_json);
        
        // 执行测试函数
        const result = testFunction.execute(args);
        
        // 返回成功响应
        callback(null, {
          status: TestStatus.PASSED,
          failure_detail: null,
          return_value_json: JSON.stringify(result)
        });
      } catch (error) {
        // 返回失败响应
        callback(null, {
          status: TestStatus.FAILED,
          failure_detail: error instanceof Error ? error.message : String(error),
          return_value_json: null
        });
      }
    }
  });
  
  return server;
}

// 启动服务器函数
export function startServer(testFunction: TestFunction, address: string = '[::1]:50051'): Promise<grpc.Server> {
  return new Promise((resolve, reject) => {
    const server = createTestServer(testFunction);
    
    server.bindAsync(address, grpc.ServerCredentials.createInsecure(), (err, port) => {
      if (err) {
        reject(err);
        return;
      }
      
      server.start();
      console.log(`Node.js Quickcheck Runner for '${testFunction.propertyName}' started on ${address}`);
      resolve(server);
    });
  });
}

// 示例测试函数：加法测试
const addTest: TestFunction = {
  propertyName: 'property_add',
  execute(args: { a: number; b: number }) {
    return args.a + args.b;
  }
};

// 示例测试函数：反转测试
const reverseTest: TestFunction = {
  propertyName: 'property_reverse',
  execute(args: { xs: number[] }) {
    return args.xs.slice().reverse();
  }
};

// 主函数 - 启动示例服务器
if (import.meta.main) {
  // 默认启动加法测试服务器
  startServer(addTest)
    .then(server => {
      console.log('Server is running. Press Ctrl+C to stop.');
      
      // 优雅关闭
      process.on('SIGINT', () => {
        server.tryShutdown(() => {
          console.log('Server stopped.');
          process.exit(0);
        });
      });
    })
    .catch(err => {
      console.error('Failed to start server:', err);
      process.exit(1);
    });
}

// 导出类型和函数供外部使用
export { TestFunction, TestStatus, createTestServer };
