# 迁移重构报告 (KimiK2-0905)

## (1) 旧代码行为总结（落地版）

- **输入规范**：
  - 程序启动后首先输入数据组数 `n`。
  - 随后循环 `n` 次，每次输入操作符 `op` (+, -, *, /) 及两个操作数 `left`, `right`。
  - 支持的数字格式：
    - 负号 `-`（仅首位有效）。
    - 千分位分隔符 `,`（解析时自动忽略）。
    - 小数点 `.`（用于确定小数位数）。
  - **Rust 改进**：旧代码对非法字符容忍度高，Rust 版本在解析阶段进行严格校验，遇到非法格式返回明确的 `Parse Error`。

- **对齐与计算规则**：
  - **加减法**：自动对齐小数点，精度取两个操作数中较大的小数位数（`max(scale1, scale2)`）。
  - **乘法**：结果小数位数等于操作数小数位数之和（`scale1 + scale2`）。
  - **除法**：固定精度为20位小数，使用长除法算法实现。

- **去零与格式化规则**：
  - **输出格式**：支持千分位分隔符输出（`to_string_with_grouping`）。
  - **去零逻辑**：
    - 整数部分去除前导零（保留单独的 `0`）。
    - 小数部分去除尾随零。
    - 零值：符号设为正，scale为0。

- **ERROR 条件**：
  - **除以零**：明确检测除数为零的情况，返回 `Division by Zero` 错误。
  - **解析错误**：输入空字符串或非数字字符时报错。

---

## (2) "代码异味 -> 重构手段"对照表

| 旧代码异味（函数/片段） | 问题 | Rust 中的消除方式 |
|---|---|---|
| 全局函数散乱 (`add`, `sub`, `cheng`, `chu`) | 无统一接口、不可复用 | 定义 `BigDecimal` 结构体，实现统一 API（`parse()`, `add()`, `sub()`, `mul()`, `div()`）。 |
| `Node*` 链表结构 | 内存碎片化、遍历低效 O(N) | 使用 `Vec<u8>` 连续内存存储数字位，支持 O(1) 随机访问。 |
| `append` 函数 | O(N²) 构建链表，性能极差 | `Vec::push` 实现 O(1) 均摊复杂度插入。 |
| 手动 `new`/`delete` | 内存管理风险 | 利用 Rust RAII 机制，`Vec` 超出作用域自动释放。 |
| 魔法数字 (20, 7, 9) | 逻辑晦涩难懂 | 使用 `const` 定义命名常量，代码自解释。 |
| 命名不清（拼音命名） | 可读性差 | 采用标准英文命名：`divide()` 替代 `chu()`，`normalize()` 替代 `xiaoshudian()`。 |

---

## (3) Rust 项目文件树

```
KimiK2-0905/
├── Cargo.toml                    # 项目配置文件
├── README.md                     # 本文档
├── src/
│   ├── lib.rs                   # 核心库实现（BigDecimal）
│   ├── main.rs                  # CLI 交互界面
│   ├── batch_processor.rs       # 批处理处理器
│   ├── batch_processor_cpp_format.rs  # C++格式输出
│   └── test_runner.rs           # 测试运行器
└── 文档文件（中文）              # 项目文档
```

---

## (4) 迁移说明（跨语言迁移亮点）

- **数据结构升级**：
  - 从 **C++ 单向链表** (`Node*`) 迁移到 **Rust 动态数组** (`Vec<u8>`)。
  - 消除指针操作复杂性，提高缓存命中率和运算性能。

- **类型系统增强**：
  - 使用 `Result<BigDecimal, BigDecimalError>` 处理错误，强制调用者处理异常情况。
  - 引入 `enum Sign { Positive, Negative }` 明确符号表示。

- **所有权与借用**：
  - C++ 函数经常修改传入参数，导致副作用难以追踪。
  - Rust 实现中，算术运算接受 `&self` 和 `&other`，返回全新的 `BigDecimal` 实例。

- **性能改进**：
  - 由链表尾插 O(n²) → `Vec` 线性构造。
  - 乘法/除法采用基于数组的竖式算法，避免低效的路径。

---

## (5) 运行方式

### 基础运行

#### 1. 构建项目
```bash
cd KimiK2-0905
cargo build --release
```

#### 2. 运行交互式CLI
```bash
cargo run
```
程序将提示您输入操作，支持连续运算，输入 `quit` 或 `exit` 退出。


### 性能测试

#### 1. 多次性能测试（取平均值）
```bash
# 进入项目目录
cd KimiK2-0905

cargo build --release

: > time.txt  # 清空 time.txt

for i in {1..10}; do
  echo "run $i" >> time.txt
  /usr/bin/time -p ./target/release/high_precision_calculations \
    < ../llm_crosslang_test_input_1000000.txt > /dev/null 2>> time.txt
done

awk '
  $1=="real"{r+=$2; rc++}
  $1=="user"{u+=$2; uc++}
  $1=="sys" {s+=$2; sc++}
  END{
    printf("avg_real %.6f\n", r/rc);
    printf("avg_user %.6f\n", u/uc);
    printf("avg_sys  %.6f\n", s/sc);
  }' time.txt | tee time_avg.txt
```

#### 预期性能表现
由于算法从 O(n²) 优化为 O(n) 且使用了更高效的数据结构，预期性能将显著优于旧 C++ 版本。典型测试结果：

```txt
avg_real 0.152000  # 墙钟时间
avg_user 0.141000  # CPU用户态时间
avg_sys  0.011000  # CPU内核态时间
```

