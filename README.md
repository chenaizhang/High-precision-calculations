# README

本项目是用来测试 LLM 在代码重构与去除代码异味的效果。

`./old_code`中包含需要重构的代码，其完成了一个链表高精度计算器，支持加减乘除。代码存在[许多问题](./old_code/代码问题.md)。本测试通过验证重构后代码的**结果一致性**、**运行速度**等指标来验证重构的效果。

- `Codex` 为 Codex Extra high 模型
- `GPT5_1` 为 cursor 默认模型
- `gemini` 为在 Trae 中使用 Gemini-3-Pro-Preview (200k) 模型
- `DeepSeek` 为在 Trae 中使用 DeepSeek-V3.1 模型
- `Grok-4` 为在 Trae 中使用 Grok-4 模型
- `KimiK2-0905` 为在 Trae 中使用 KimiK2-0905 模型

## 指标

### 一、验证 C++ 和 Rust 的结果一致性指标

#### 0）先用小样本验证

用 小测试文件 先跑通对比流程：

```bash
# 假设当前目录有 calc 和 Codex/
INPUT=llm_crosslang_test_input.txt
```

跑通后再换成 `llm_crosslang_test_input_1000000.txt`。

#### 1）分别跑出原始输出

##### C++：

```bash
cd old_code
g++ -O2 -DNDEBUG -std=c++17 main.cpp -o calc
cd ..

./old_code/calc < "$INPUT" > ./output/out_cpp.txt
echo $?
```

##### Rust：

```bash
cd <路径>
cargo build --release
cd ..

./<路径>/target/release/high_precision_calculations < "$INPUT" > ./output/out_rust.txt
echo $?
```

> `echo $?` 两边都应为 `0`，否则先修崩溃/异常退出。

#### 2）对比内容：先用哈希快速判断“完全一致”

```bash
cd output
sha256sum out_cpp.txt out_rust.txt
cd ..
```

两个 hash 一样， 则**结果完全一致**。

### 二、验证 C++ 和 Rust 的运行速度指标

参照每个项目文件夹的`README.md`文件的运行方式模块，运行相应的代码，最后能得到相应的`time_avg.txt`文件，最后可以与`./old_code/time_avg.txt`里的结果做对比。

## 结论

| 模型                        | 结果是否正确 |
| --------------------------- | ------------ |
| old_code                    | ✔            |
| Codex Extra high            | ✔            |
| GPT5_1                      | ✔            |
| Gemini-3-Pro-Preview (200k) | ✔            |
| Grok-4                      | ✔            |
| KimiK2-0905                 | ❌           |
| DeepSeek-V3.1               | ✔            |

| 模型                        | 1,000，000 次平均计算耗时(s) |
| --------------------------- | ---------------------------: |
| old_code                    |                       26.939 |
| Codex Extra high            |                        2.214 |
| GPT5_1                      |                        2.627 |
| Gemini-3-Pro-Preview (200k) |                        1.934 |
| Grok-4                      |                        2.361 |
| KimiK2-0905                 |                        10.09 |
| DeepSeek-V3.1               |                        1.608 |

- `Codex Extra high` 与 `GPT5_1` 只花费了一次上下文次数，且代码结果执行正确，效果最还原，未发现无明显漏洞，最终**1,000，000 次平均计算耗时(s)**有显著下降。
- `Gemini-3-Pro-Preview (200k)` 第一次上下文中出现了错误，再手动进行 提示词 debug 后在第六次上下文中错误得到了解决，**1,000，000 次平均计算耗时(s)**有显著下降。
- `Grok-4` 第一次上下文中出现了错误，自行 debug 八次，手动 debug 三次后代码执行正确，最终的**1,000，000 次平均计算耗时(s)**有显著下降。
- `KimiK2-0905` 第一次上下文中出现了错误，且花费了六次上下文进行了提示词 debug，但最后依然有明显的错误（输出格式错误），**1,000，000 次平均计算耗时(s)**比其他模型显著的差。
- `DeepSeek-V3.1` 第一次上下文出现了错误，四次手动提示词 debug 后问题得到解决，最终的**S1,000，000 次平均计算耗时(s)**比 `old_code` 也有显著下降。

## 有趣的事情

1. 有时候 TRAE 的 AI 侧栏会出现一直加载不出来的情况，发现只要重启 WSL 问题就消失了。
2. TRAE 的 AI 终端经常自己卡死，删都删不掉。
