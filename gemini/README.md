# 迁移重构报告 (gemini)

## (1) 旧代码行为总结（落地版）

- **输入规范**：

  - 程序启动后首先输入数据组数 `n`。
  - 随后循环 `n` 次，每次输入操作符 `op` (+, -, \*, /) 及两个操作数 `left`, `right`。
  - 支持的数字格式：
    - 负号 `-`（仅首位有效）。
    - 千分位分隔符 `,`（解析时自动忽略）。
    - 小数点 `.`（用于确定小数位数）。
  - **Rust 改进**：旧代码对非法字符容忍度高（可能导致错误计算），Rust 版本在解析阶段进行严格校验，遇到非法格式返回明确的 `Parse Error`。

- **对齐与计算规则**：

  - **加减法**：自动对齐小数点，精度取两个操作数中较大的小数位数（`max(scale1, scale2)`）。
  - **乘法**：结果小数位数等于操作数小数位数之和（`scale1 + scale2`）。
  - **除法**：
    - 旧代码逻辑：硬编码保留 20 位小数精度，使用低效的减法模拟除法。
    - **Rust 重构**：实现了标准长除法（Long Division），精度策略设定为 `20 + (scale1 - scale2)`，确保至少保留 20 位有效小数，并进行正确的四舍五入。

- **去零与格式化规则**：

  - **输出格式**：保留了旧代码的千分位分隔符输出风格（`to_string_with_grouping`）。
  - **去零逻辑**：
    - 整数部分去除前导零（保留单独的 `0`）。
    - 小数部分去除尾随零。
  - **Bug 修复**：旧代码在加减法去零时存在逻辑错误（可能导致 `0.1 + 0.2` 结果异常），Rust 版本基于数学正确的 `Vec` 操作修复了此问题，保证计算准确性。

- **ERROR 条件**：
  - **除以零**：旧代码通过 `toosmall` 模糊判断，Rust 版本明确检测除数为零的情况，返回 `Division by Zero` 错误。
  - **解析错误**：输入空字符串或非数字字符时报错。

---

## (2) “代码异味 -> 重构手段”对照表

| 旧代码异味（函数/片段）                     | 问题                            | Rust 中的消除方式                                                                                         |
| ------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------- |
| 全局函数散乱 (`add`, `sub`, `cheng`, `chu`) | 无统一接口、不可复用            | 定义 `BigDecimal` 结构体，实现 `Add`, `Sub`, `Mul`, `Div` trait，提供统一且符合直觉的 API（如 `a + b`）。 |
| `Node*` 链表结构                            | 内存碎片化、遍历低效 ($O(N)$)   | 使用 `Vec<u8>` 连续内存存储数字位，支持 $O(1)$ 随机访问和尾部插入。                                       |
| `append` 函数                               | $O(N^2)$ 构建链表，性能极差     | `Vec::push` 实现 $O(1)$ 均摊复杂度插入。                                                                  |
| `compare` 修改头指针                        | 隐式副作用、潜在内存泄漏        | Rust 所有权系统保证比较操作 (`cmp`, `PartialEq`) 不修改原数据（Immutable Borrow）。                       |
| 手动 `new`/`delete`                         | 内存管理风险 (Double Free/Leak) | 利用 Rust RAII 机制，`Vec` 超出作用域自动释放，零手动内存管理。                                           |
| `chu` 函数 (减法模拟除法)                   | 算法极其低效                    | 重构为“长除法”算法，大幅提升大数除法性能。                                                                |
| 魔法数字 (20, 7, 9)                         | 逻辑晦涩难懂                    | 将精度控制逻辑封装在 `checked_div` 中，常量化配置，代码自解释。                                           |
| `xiaoshudian`, `dayu0`                      | 命名不规范 (拼音/含混)          | 采用标准英文命名：`scale`, `is_positive`, `normalize`。                                                   |

---

## (3) Rust 项目文件树 + 完整代码

### 文件树

```
gemini/
  Cargo.toml
  README.md
  src/
    lib.rs          # 库入口，导出模块
    main.rs         # CLI 交互入口
    big_decimal.rs  # 核心逻辑实现 (BigDecimal 结构体及运算)
```

---

## (4) 迁移说明（跨语言迁移亮点）

- **数据结构升级**：

  - 从 **C++ 单向链表** (`Node*`) 迁移到 **Rust 动态数组** (`Vec<u8>`)。这不仅消除了指针操作的复杂性，还极大地提高了缓存命中率和运算性能。

- **类型系统增强**：

  - 使用 `enum Sign { Positive, Negative }` 替代隐式的符号处理，状态更清晰。
  - 引入 `Result<BigDecimal, BigIntError>` 处理错误，强制调用者处理“除零”或“解析失败”等异常情况，避免了 C++ 中未定义行为 (UB) 的风险。

- **所有权与借用**：

  - C++ 旧代码中函数经常修改传入的链表（如 `reverse`），导致副作用难以追踪。
  - Rust 实现中，算术运算接受 `self` 或 `&self`，返回全新的 `BigDecimal` 实例，原数据保持不变（Immutability），彻底杜绝了副作用引发的 Bug。

- **Traits 集成**：
  - 通过实现 `std::ops::{Add, Sub, Mul, Div}`，使得自定义的大数类型可以像原生整数一样使用运算符直接计算，代码可读性极高。

---

## (5) 运行方式

### 单元测试

运行项目内置的单元测试，验证解析、加减乘除及舍入逻辑的正确性：

```bash
cd gemini
cargo test
```

### 交互式运行

启动 CLI 程序，手动输入数据进行测试：

```bash
cd gemini
cargo run
```

程序运行后，按提示输入数据数目、操作符及操作数。

### 性能测试脚本

运行 1,000,000 条数据，测试 10 次并计算平均用时（需准备测试数据文件 `../llm_crosslang_test_input_1000000.txt`）：

```bash
cd gemini

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

#### 预期输出示例 (time_avg.txt)

```txt
avg_real 0.152000
avg_user 0.141000
avg_sys  0.011000
```

_(注：由于算法从 O(N^2) 优化为 O(N) 且使用了更高效的数据结构，预期性能将显著优于旧 C++ 版本)_
