# 迁移重构报告（DeepSeek）

## (1) 旧代码行为总结（落地版）

- **输入规范**：
  - 交互读取 `n`，每次读取 `op`、`left`、`right`，用 `>>` 作为分隔，等价于"按空白分隔的 token"。
  - 数字字符串允许 `-`（仅首位有效）、`,`（忽略）、`.`（仅用于统计小数位）。
  - 非数字字符（如多余的 `-`）在旧代码中不会报错，可能被当作数字处理；Rust 版本对非法字符返回错误，并在 CLI 中输出 `ERROR`。

- **对齐规则（加/减）**：
  - `xiaoshudian()` 统计 `.` 后位数，`main` 里通过**在字符串末尾补 `0`**对齐小数位（不是在内部结构里补）。

- **去零规则**：
  - `delete0(result, x)` 在"低位在表头"的链表上移除**小数尾随 0**，并相应减小 `x`。
  - 随后对"高位在表头"的链表再次 `delete0`，用于去掉整数部分前导 0。
  - **关键 bug**：加/减时使用 `delete0(result, length-x)`，导致当结果在 (-1,1) 时会删除整数位的 `0`，从而输出错误（例：`0.1 + 0.2 -> 3`）。
  - 乘/除时使用 `delete0(result, length-x-1)`，保留了整数 `0`，因此 `0.125` 会正常显示。

- **乘除的小数与舍入规则**：
  - 乘法结果小数位数 `x = x1 + x2`；除法内部**固定计算 20 位小数**（`chu()` 内部写死），`main` 再用 `x = 20 + x1 - x2` 解释小数点位置。
  - `print_2()` 对小数位**最多输出 10 位**，第 11 位用于"末位四舍五入"。
  - 舍入有 bug：只对最后一位 `+1`，**不做进位传播**（如 `0.99999999995` 输出 `0.99999999910`）。

- **ERROR 条件**：
  - `toosmall()` 将除数反转后检查"最高 7 位是否全为 0"，若是则输出 `ERROR`。
  - 这等价于把 `|divisor| < 1e-6`（且无额外前导零）当作除 0；有前导零会被误判。

- **符号相关不一致**：
  - 负号以"**每一位都是负数**"方式存储。
  - 乘/除结果若绝对值 < 1，会因为首位是 0 而**丢失负号**（`0.1 * -0.2 -> 0.02`）。
  - Rust 版本内部用独立 `sign`，但**在 legacy 输出层复刻该异常行为**。

- **潜在 bug / 不一致点（并在 Rust 中避免但复刻外部行为）**：
  - `compare(Node*& ...)` 修改头指针导致内存泄漏和副作用；Rust 用值/借用比较，**不破坏输入**，但输出仍与旧行为一致。
  - `subforchu()` 有空指针风险；Rust 版本不再原地修改，消除 UB。
  - `print_2()` 四舍五入不进位；Rust 版本在 legacy formatter 中**保留该输出错误**，但核心数值保持正确。

---

## (2) "代码异味 -> 重构手段"对照表

| 旧代码异味（函数/片段） | 问题 | Rust 中的消除方式 |
|---|---|---|
| 全局函数 + `Node*` 传来传去 | 无统一接口、难复用 | 统一为 `BigDecimal` 类型，方法化 API（`parse/add/sub/mul/div/to_string_*`），见 `DeepSeek/src/lib.rs` |
| `compare(Node*& ...)` / `negation()` | 隐式副作用、破坏输入 | 值语义 + 不可变借用比较，不修改参数；输出保持一致 |
| `chu()` 内部正负两大分支重复 | 重复代码、维护困难 | `div()` 先做符号剥离，核心算法单分支，统一结果符号 |
| `append()` O(n²) 构造链表 | 性能陷阱 | 改为 `Vec<u8>` 直接 push，批量扩容 |
| `reverse()` 频繁反转 | 多余遍历 | 基于"高位在前"的 `Vec`，只在算法内反向索引 |
| `toosmall()` / `print_2()` 的 7/9/20 | 魔法数字 | 提升为 `const`（`TOO_SMALL_PREFIX_DIGITS/DIV_FRACTIONAL_DIGITS/LEGACY_ROUND_DIGITS`） |
| 裸指针 + `new/delete` | 内存安全风险 | Rust 所有权 + 自动释放（无手动 `delete`） |
| `dayu0` / `ispositive` 等命名混乱 | 可读性差 | 语义化命名：`sign/is_zero/is_too_small_divisor` |
| `subforchu()` 原地修改 | 不可预测副作用 | 纯函数式 `div_digits` + `sub_digits`，无输入修改 |

---

## (3) Rust 项目文件树 + 完整代码

### 文件树
```
DeepSeek/
  Cargo.toml
  README.md
  src/
    lib.rs
    main.rs
  tests/
    legacy_behavior.rs
```

---

## (4) 迁移说明（跨语言迁移亮点）

- **链表 + 裸指针 → Vec + 所有权**：
  - C++ 的 `Node*` 版本需要手动 `new/delete`、频繁 `reverse`，并存在 `compare` 改头指针导致泄漏的问题。
  - Rust 版改为 `Vec<u8>` 存放高位到低位，所有权明确；函数均以不可变借用读取、返回新值，无隐式副作用。

- **符号与数值分离**：
  - C++ 用"每一位数字的正负"表示符号，导致 `ispositive/dayu0` 等函数绕且脆弱。
  - Rust 用 `sign + digits + scale` 三元建模，`normalize()` 强制保证"不变量"（无多余前导零、零值无负号）。

- **算法与格式化解耦**：
  - 旧代码把运算逻辑和格式化纠缠在一起（`delete0 + print_1/print_2`）。
  - Rust 把运算结果与输出格式拆开：核心运算给出**正确值**，legacy 格式层再复刻旧输出细节（包括 bug），保证"外部行为一致"。

- **类型系统强制不变量**：
  - `BigDecimal` 的 `digits` 永远是 0..=9；`normalize()` 在边界时统一归一化为 `0`，从源头避免空链表/空指针风险。
  - `Result` 明确标注除法错误（`DivisionByZeroOrTooSmall`），使调用者必须处理。

- **性能改进**：
  - 由链表尾插 O(n²) → `Vec` 线性构造。
  - 乘法/除法采用基于数组的竖式与长除法，避免 `chu()` 的"反复减法"极慢路径。

## (5) 运行方式

### 测试

```bash
cd DeepSeek
cargo test
```

### 交互运行

```bash
cd DeepSeek
cargo run
```

输入格式示例：
```
1
+ 123.456 789.123
```

### 运行1,000,000条数据，测试10次，记录平均用时

```bash
cd DeepSeek
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

#### 示例返回位于`time_avg.txt`

```txt
avg_real 2.214000 // 墙钟时间（从开始到结束你实际等了多久）
avg_user 1.815000 // CPU 在用户态执行你程序代码的时间（算法、循环、计算）
avg_sys  0.393000 // CPU 在内核态花的时间（系统调用/IO/内存分配/释放/页缓存等）
```
