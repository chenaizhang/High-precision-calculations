# README

## 验证 C++ 和 Rust 的结果一致性指标

### 0）先用小样本验证

用 小测试文件 先跑通对比流程：

```bash
# 假设当前目录有 calc 和 Codex/
INPUT=llm_crosslang_test_input.txt
```

跑通后再换成 `llm_crosslang_test_input_1000000.txt`。

### 1）分别跑出原始输出

#### C++：

```bash
cd old_code
g++ -O2 -DNDEBUG -std=c++17 main.cpp -o calc
cd ..

./old_code/calc < "$INPUT" > ./output/out_cpp.txt
echo $?
```

#### Rust：

```bash
cd <路径>
cargo build --release
cd ..

./<路径>/target/release/high_precision_calculations < "$INPUT" > ./output/out_rust.txt
echo $?
```

> `echo $?` 两边都应为 `0`，否则先修崩溃/异常退出。

### 2）对比内容：先用哈希快速判断“完全一致”

```bash
cd output
sha256sum out_cpp.txt out_rust.txt
cd ..
```

两个 hash 一样， 则**结果完全一致**。
