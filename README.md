# README

本项目是用来测试 LLM 在代码重构与去除代码异味的效果。`./old_code`测试所用的代码，其作用是完成了一个链表高精度计算器，支持加减乘除。代码存在[许多问题](./old_code/代码问题.md)。测试通过验证重构后代码的**结果一致性**、**运行速度**、**代码可复用性**等指标来验证重构的效果。

## 一、验证 C++ 和 Rust 的结果一致性指标

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

## 二、验证 C++ 和 Rust 的运行速度指标

参照每个项目文件夹的`README.md`文件的运行方式模块，运行相应的代码，最后能得到相应的`time_avg.txt`文件，最后可以与`./old_code/time_avg.txt`里的结果做对比。
