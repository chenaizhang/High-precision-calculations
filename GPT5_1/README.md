## 项目介绍（GPT5_1）

本目录是基于旧 C++ 链表实现的“高精度十进制四则运算”在 Rust 下的**迁移 + 重构版本**，由 GPT-5.1 生成。

- **核心类型**：`BigDecimal`（位于 `src/lib.rs`）
  - `sign: Sign`：正负号
  - `digits: Vec<u8>`：十进制位，**大端**存储（高位在前）
  - `scale: i32`：小数位数
  - `fn parse(input: &str) -> Result<BigDecimal, Error>`
  - `fn to_string_with_grouping(&self, grouping: bool) -> String`
  - `fn add/sub/mul(&self, other: &Self) -> Self`
  - `fn div(&self, other: &Self) -> Result<Self, Error>`
- **旧行为格式化**：通过 `LegacyFormat` + `to_string_legacy` 复刻 C++ `print_1/print_2` 的输出（包括千分位、小数位数控制、“ERROR” 规则以及历史舍入 bug）。
- **CLI 入口**：`src/main.rs`，交互提示与输入协议保持与 `old_code/main.cpp` 一致。

### 文件结构

- `Cargo.toml`：Rust crate 配置
- `src/lib.rs`：高精度十进制核心实现
- `src/main.rs`：命令行交互入口
- `旧代码行为总结.md`：对 C++ 旧实现外部行为的归纳
- `代码异味_重构对照表.md`：旧代码异味与本次重构手段对照
- `迁移说明.md`：跨语言迁移的设计要点与不变量说明

## 运行方式

### 交互式运行

```bash
cd GPT5_1
cargo run
```
在输入完所有数据后输入EOF终止输入流。

### 运行1,000,000条数据，测试10次，记录平均用时

```bash
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
avg_real 2.627000 // 墙钟时间（从开始到结束你实际等了多久）
avg_user 1.795000 // CPU 在用户态执行你程序代码的时间（算法、循环、计算）
avg_sys  0.773000 // CPU 在内核态花的时间（系统调用/IO/内存分配/释放/页缓存等）
```


