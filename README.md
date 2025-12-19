# README

验证 C++ 和 Rust 的“运行结果一模一样”**，最靠谱的做法是：

1. 用同一份输入文件跑两边
2. 把两边输出里“提示语”剥掉，只留下每条 case 的**结果字符串**
3. 对比两份结果文件（先对比行数，再 diff / hash）
4. 如果不一样，自动定位到第几条 case 出错

下面给你一套可以直接复制跑的命令（支持 100 万条）。

---

## 0）建议先用小样本验证（避免 1e6 级别 debug 太痛）

用 quick 文件（或你自己缩小 n）先跑通对比流程：

```bash
# 假设当前目录有 calc 和 Codex/
INPUT=llm_crosslang_test_input_quick.txt
```

跑通后再换成 `llm_crosslang_test_input_1000000.txt`。

---

## 1）分别跑出原始输出（包含中文提示语）

### C++：

```bash
./calc < "$INPUT" > out_cpp_raw.txt
echo $?
```

### Rust（release 可执行文件更稳定；在 Codex/ 外部也能直接跑）：

```bash
cd Codex
cargo build --release
cd ..

./Codex/target/release/high_precision_calculations < "$INPUT" > out_rust_raw.txt
echo $?
```

> `echo $?` 两边都应为 `0`，否则先修崩溃/异常退出。

---

## 2）把输出“提取成纯结果行”

你的两份程序都会把提示语和结果拼在同一行（因为提示是 `print!`/`cout<<` 没换行），所以用 `sed` 取**最后一个中文冒号 `：` 后面的内容**最稳：

```bash
sed 's/.*：//' out_cpp_raw.txt  | tr -d '\r' > out_cpp_results.txt
sed 's/.*：//' out_rust_raw.txt | tr -d '\r' > out_rust_results.txt
```

现在 `out_*_results.txt` 每行应该只剩下：`ERROR` 或 `-0-9.,` 组成的结果。

---

## 3）先对比“行数是否等于 n”

（注意：n=1000000 时输出结果应该正好 1000000 行）

```bash
head -n 1 "$INPUT"
wc -l out_cpp_results.txt out_rust_results.txt
```

如果行数不等：

* 有一边提前退出了（或遇到 parse error 输出 ERROR 后还继续没问题，但不应该少行）
* 或你的提取规则不对（极少见）

---

## 4）对比内容：先用哈希快速判断“完全一致”

```bash
sha256sum out_cpp_results.txt out_rust_results.txt
```

两个 hash 一样 ⇒ **结果完全一致**。

不一样 ⇒ 继续第 5 步定位差异。

---

## 5）定位第一处不一致（能直接告诉你第几条 case）

用这个 Python 脚本“流式对比”（不会把 100 万行全读入内存）：

```bash
python3 - <<'PY'
inp = "llm_crosslang_test_input_1000000.txt"   # 需要的话改成 quick
cpp = "out_cpp_results.txt"
rst = "out_rust_results.txt"

with open(inp, "r", encoding="utf-8") as fi, \
     open(cpp, "r", encoding="utf-8") as fc, \
     open(rst, "r", encoding="utf-8") as fr:

    n_line = fi.readline()
    n = int(n_line.strip())
    for i in range(1, n+1):
        case = fi.readline().rstrip("\n")
        oc = fc.readline()
        oru = fr.readline()
        if oc == "" or oru == "":
            print(f"输出行数不足：在第 {i} 条时遇到 EOF")
            break
        oc = oc.rstrip("\n")
        oru = oru.rstrip("\n")
        if oc != oru:
            print(f"发现不一致：第 {i} 条")
            print("输入：", case)
            print("C++ ：", oc)
            print("Rust：", oru)
            break
    else:
        # 还要检查有没有多出来的行
        extra_cpp = fc.readline()
        extra_rst = fr.readline()
        if extra_cpp or extra_rst:
            print("两边有额外输出行（超过 n）。")
        else:
            print("✅ 全部匹配：", n, "条")
PY
```

---

## 6）把流程一键化（推荐）

你把 `INPUT` 改成你要测的文件即可：

```bash
INPUT=llm_crosslang_test_input_1000000.txt

./calc < "$INPUT" > out_cpp_raw.txt
./Codex/target/release/high_precision_calculations < "$INPUT" > out_rust_raw.txt

sed 's/.*：//' out_cpp_raw.txt  | tr -d '\r' > out_cpp_results.txt
sed 's/.*：//' out_rust_raw.txt | tr -d '\r' > out_rust_results.txt

sha256sum out_cpp_results.txt out_rust_results.txt || true
```

---

### 小提醒（很关键）

你 Rust 版 **parse 更严格**、旧 C++ 对一些“怪字符”可能不报错，这会导致某些输入下结果不一致。你现在这份 `llm_crosslang_test_input_1000000.txt` 只含 `- , . 0-9`，理论上不触发 parse 差异；如果你后续换了输入生成器，要注意这一点。

如果你愿意，把你两边 `sha256sum` 的输出贴出来（或第 5 步定位到的第一条不一致 case），我可以直接帮你判断：是 legacy quirks 没对齐（如 toosmall/舍入/丢符号），还是提取规则导致的误差。
