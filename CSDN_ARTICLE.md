# Rust + Ratatui 实战：从零构建一个 JSON 终端编辑器，已开源

## 前言

一个月前我刚开始学 Rust，语法都还搞不清。因为对终端工具有兴趣，就照着 Ratatui 的官方教程写了个计数器 demo，发现 TUI 这玩意儿挺有意思的。

后来在 GitHub 上看到 [csvlens](https://github.com/YS-L/csvlens)——一个 CSV 文件的终端查看器。我就在想：**JSON 文件是不是也缺这么一个工具？**

平时工作中经常要改各种配置文件，有时候在服务器上只能 vim 硬改，JSON 一长就眼花。于是决定自己写一个：既能查看 JSON 结构，又能在终端里直接编辑的 TUI 工具。

折腾了一个月，从 Rust 语法到 Ratatui 组件到事件处理到 GitHub Actions 自动构建，终于搞出了第一个能用的版本：[json-tui](https://github.com/xxxhyy9090/json-tui)。

## 项目简介

json-tui 是一个终端 JSON 文件查看和编辑工具，主要有这几个功能：

- 启动后自动扫描当前目录的 `.json` 文件，用光标选择打开
- 树形视图浏览深层嵌套的对象，支持折叠展开
- 表格视图浏览数组数据，支持单元格导航和编辑
- 修改值、删除节点、搜索、保存一步到位
- 支持传目录参数或直接传文件路径，也能在工具内用 `:e` 跳转
- 跨平台，Windows / Linux / macOS 都能跑

## 截图

（这里放几张运行截图，tree view 和 table view 各一张）

## 技术栈

没啥花里胡哨的依赖，就三个核心 crate：

- [ratatui](https://crates.io/crates/ratatui) 负责整个终端 UI
- [crossterm](https://crates.io/crates/crossterm) 处理终端的跨平台和键盘事件
- [serde_json](https://crates.io/crates/serde_json) 负责 JSON 的解析和序列化

配色用了 Nord 主题，相比终端的默认高亮色舒服很多。

## 开发过程踩过的坑

### 1. 所有权和借用的折磨

Rust 新手最大的槛。写事件循环的时候，`terminal.draw()` 需要一个闭包借用 `app`，而 `event::read()` 之后又要 `&mut app` 改状态——编辑器天天报"cannot borrow as mutable because it is also borrowed as immutable"。

后来理解了：**draw 只读，event 处理要写，画完再处理事件就不会冲突**。核心结构就是：

```rust
loop {
    terminal.draw(|f| draw_ui(f, &app))?;  // 只读借用
    if let Event::Key(key) = event::read()? {
        handle_key(key, &mut app);           // 可变借用，上一个借用已释放
    }
}
```

### 2. UTF-8 字符边界

编辑框里输入中文直接 panic 崩溃，报错信息是"byte index is not a char boundary"。原因是我用 `cursor_pos += 1` 来移动光标，但中文字符占 3 个字节，光标会卡在字符中间。改成 `c.len_utf8()` 和 `is_char_boundary()` 才解决。

### 3. 扁平化 JSON 树

JSON 天然是树形结构，但 TUI 渲染是行式结构。我的做法是把 `serde_json::Value` 递归遍历成一维 Vec，每个节点存 depth、expanded、child_count。折叠展开时直接跳过子树，不用重建数据。

### 4. GitHub Actions 打包路径

CI 脚本里打包 README 写了个相对路径，Windows 能跑但 Unix 跑不了。犯了个低级错误，改了两行就好了。

## 使用说明

### 安装

下载 [Releases](https://github.com/xxxhyy9090/json-tui/releases) 里的二进制，解压直接运行，不需要装 Rust。

或者用 Cargo：

```bash
cargo install json-tui
```

源码编译：

```bash
git clone https://github.com/xxxhyy9090/json-tui.git
cd json-tui
cargo build --release
```

### 基本用法

```bash
# 扫描当前目录的 JSON 文件，用光标选
json-tui

# 扫描指定目录
json-tui ~/my-configs/

# 直接打开某个文件
json-tui ~/configs/database.json
```

### 操作指南

**文件选择器**

| 按键 | 操作 |
|------|------|
| `j/k` 或 `↑↓` | 上下选文件 |
| `Enter` | 打开选中的文件 |
| `:e /path` + Enter | 打开其他路径的文件或目录 |
| `r` | 刷新列表 |
| `q` | 退出 |

**JSON 查看器**

| 按键 | 操作 |
|------|------|
| `j/k` `↑↓` | 上下移动 |
| `h/l` `←→` | 折叠/展开节点（树形）或移动列（表格） |
| `g/G` | 跳到顶部/底部 |
| `Enter` | 编辑当前值或单元格 |
| `d` | 删除节点或行 |
| `Tab` | 切换视图（树形/表格/自动） |
| `1/2/3` | 强制切换到树形/表格/自动视图 |
| `/` | 搜索，`n/N` 跳转匹配 |
| `:w` + Enter | 保存文件 |
| `q` | 返回文件选择器（有未保存修改会提示） |

**编辑模式**

| 按键 | 操作 |
|------|------|
| 直接输入 | 键入新值 |
| `Enter` | 确认 |
| `Esc` | 取消 |
| `← → Home End Backspace Delete` | 光标操作 |

## 待完善

目前还有几个想做的功能：

- 撤销/重做
- 拖拽式的多文件支持
- 更多主题可选
- 表格编辑的体验优化

## 结尾

这个项目是我学 Rust 一个月以来的第一个成品，写得比较糙，但功能基本够用了。如果你也有在终端里改 JSON 的需求，可以试试看。

项目地址：[github.com/xxxhyy9090/json-tui](https://github.com/xxxhyy9090/json-tui)

有问题或者建议欢迎提 Issue，也可以直接发邮件给我 qw061220@outlook.com。

感谢支持 (´･ω･`)
