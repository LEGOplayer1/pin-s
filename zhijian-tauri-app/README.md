# 纸间 · 便利贴（Tauri 版）

> 「纸间」——桌面级的精致极简便利贴。
> 基于 Tauri v2 构建，包体小、启动快、内存低。

---

## ✨ 功能

- **多窗口便利贴**：每张便利贴是一个独立的无边框透明窗口
- **6 色莫兰迪配色**：米白 / 雾蓝 / 淡粉 / 草绿 / 薰衣草 / 鹅黄 —— 每张便利贴可随时切换
- **自由拖拽 & 调整大小**：顶部拖动 / 右下角系统原生 resize 手柄
- **置顶**：一键让某张便利贴永远在最前面
- **Markdown 轻量格式**：标题 / 粗体 / 斜体 / 代码 / 引用
- **本地持久化**：`notes.json` 自动保存到 `%APPDATA%\zhijian`
- **系统托盘**：右键菜单可 新建 / 显示 / 隐藏 / 退出；左键单击一键切换
- **单实例**：再次启动 = 再建一张便利贴而不是重复进程
- **快捷键**：`Ctrl+N` 新建，`Ctrl+W` 关闭

---

## 📁 目录结构

```
zhijian-tauri-app/
├── docs/                        # 产品与架构文档
│   ├── 01-产品项目整体描述.md
│   └── 02-已实现功能与产品功能架构.md
├── src/                          # 前端（Webview 内容）
│   └── index.html                # 单张便利贴 UI
├── src-tauri/
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # 构建配置 + 权限 + 打包格式
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs               # 主入口 / 命令 / 托盘 / 恢复窗口
│   │   ├── note.rs               # Note 数据结构 + notes.json 读写
│   │   └── window_manager.rs     # 便利贴窗口工厂 + 状态管理
│   └── icons/                    # 图标（32x32 / 128x128 / icon.ico 等）
└── README.md
```

---

## 🚀 开发与构建

### 环境准备（Windows）

1. **Rust 工具链**（必需）
   ```
   winget install Rustlang.Rustup
   rustup default stable-x86_64-pc-windows-msvc
   ```

2. **WebView2** —— Win11 自带；Win10 请安装：<https://developer.microsoft.com/en-us/microsoft-edge/webview2/>

3. **Node.js（建议 18+）** —— 用于安装 `@tauri-apps/cli`

### 安装依赖

```bash
cd zhijian-tauri-app
npm install -D @tauri-apps/cli
# 或使用 pnpm / yarn
```

### 开发运行

```bash
npx tauri dev
```

首次运行会下载 Rust 依赖、编译后端，然后启动 WebView 窗口。后续启动几乎瞬间完成。

### 生产构建（生成安装包）

```bash
npx tauri build
```

产物位于 `src-tauri/target/release/bundle/`，包含：

- `纸间便利贴_0.1.0_x64-setup.exe`（NSIS 安装包，可选桌面快捷方式）
- `纸间便利贴_0.1.0_x64-portable.exe`（免安装便携版，双击即用）

---

## 🎨 自定义

### 更换默认颜色

编辑 `src/index.html` 中的 CSS 变量：

```css
--note-cream:    #FAF6EE;
--note-blue:     #E8EDF2;
...
```

### 调整默认窗口大小

编辑 `src-tauri/src/window_manager.rs` 中的 `DEFAULT_W` / `DEFAULT_H`。

### 更换图标

把你的 `icon.png`（≥ 512×512）放进 `src-tauri/icons/`，然后执行：

```bash
npx tauri icon src-tauri/icons/icon.png
```

Tauri CLI 会自动生成所有需要的尺寸（32 / 128 / @2x / .ico / .icns）。

---

## 🔌 IPC 命令列表

前端（TypeScript / JavaScript）调用方式：

```ts
import { invoke } from '@tauri-apps/api/core';

await invoke('cmd_create_note', { color: 'cream', content: 'hello' });
await invoke('cmd_pin_note', { pinned: true });
await invoke('cmd_save_note', { content, plainText, color });
await invoke('cmd_close_note');
await invoke('cmd_show_all');
await invoke('cmd_hide_all');
await invoke('cmd_quit_app');
await invoke('cmd_notify', { title: '提醒', body: '该喝水啦' });
```

完整命令定义请见 `src-tauri/src/main.rs`（以 `cmd_` 前缀的函数）。

---

## 🗂 数据位置

Windows：
```
%APPDATA%\zhijian\notes.json
```

示例内容：
```json
{
  "notes": [
    {
      "id": "n12a3bc45",
      "color": "cream",
      "content": "<h3>示例</h3>...",
      "plain_text": "示例",
      "pinned": false,
      "click_through": false,
      "rect": { "x": 120, "y": 140, "width": 280, "height": 280 },
      "created_at": 1712345678000,
      "updated_at": 1712345690000
    }
  ]
}
```

你可以直接备份 / 迁移这个文件；应用启动时会自动从中恢复所有窗口。

---

## 🛠 常见问题

**Q：打包时提示 "icon.ico 不存在"？**  
A：执行 `npx tauri icon src-tauri/icons/icon.png`（先放一张 ≥512×512 的 icon.png 到 icons 目录）。

**Q：npm install 很慢？**  
A：为 Rust crates.io 配置镜像：`~/.cargo/config.toml` 中添加 `replace-with = 'rsproxy'` 指向国内镜像；或使用 `cargo install --git` 时走代理。

**Q：Win10 运行时提示缺少 WebView2？**  
A：安装 Evergreen Bootstrapper：<https://go.microsoft.com/fwlink/p/?LinkId=2124703>

**Q：窗口透明效果在 Win10 上不理想？**  
A：Win10 的 WebView2 对透明背景的支持不如 Win11；建议在 Win11 下体验以获得最佳观感。

---

## 📌 下一步规划（v0.2）

- 🔎 全局搜索（跨便利贴检索内容）
- ⏰ 提醒时间 + 系统通知
- 🎯 聚焦模式（点击后其他便利贴自动淡出）
- 🧲 鼠标穿透模式（桌面水印便签）
- 🔁 开机自启

---

## 📜 许可

MIT License —— 你可以自由使用、修改、分发、商业化本项目。
