# 纸间 · 便利贴（Tauri 版）

> 「纸间」——桌面级的精致极简便利贴。
> 基于 Tauri v2 构建，包体小、启动快、内存低。

---

## 功能

- **多窗口便利贴**：每张便利贴是一个独立的无边框透明窗口
- **6 色莫兰迪配色**：米白 / 雾蓝 / 淡粉 / 草绿 / 薰衣草 / 鹅黄
- **自由拖拽 & 调整大小**：顶部/底部拖动 / 右下角系统原生 resize 手柄
- **置顶**：一键让某张便利贴永远在最前面
- **Markdown 轻量格式**：标题 / 粗体 / 斜体 / 代码 / 引用
- **本地持久化**：`notes.json` 自动保存到 `%APPDATA%\zhijian`
- **系统托盘**：右键菜单可 新建 / 显示 / 隐藏 / 退出
- **单实例**：再次启动 = 再建一张便利贴而不是重复进程
- **快捷键**：`Ctrl+N` 新建，`Ctrl+W` 关闭

---

## 目录结构

```
zhijian-tauri-app/
├── docs/                        # 产品与架构文档
├── src/                          # 前端（Webview 内容）
│   └── index.html                # 单张便利贴 UI
├── src-tauri/
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # 构建配置 + 权限 + 打包格式
│   ├── src/
│   │   ├── main.rs               # 主入口 / 命令 / 托盘 / 恢复窗口
│   │   ├── note.rs               # Note 数据结构 + notes.json 读写
│   │   └── window_manager.rs     # 便利贴窗口工厂 + 状态管理
│   └── icons/                    # 图标
└── README.md
```

---

## 开发与构建

### 环境准备（Windows）

1. **Rust 工具链**：`winget install Rustlang.Rustup`
2. **WebView2**：Win11 自带；Win10 请安装
3. **Node.js（建议 18+）**

### 开发运行

```bash
cd zhijian-tauri-app
npm install -D @tauri-apps/cli
npx tauri dev
```

### 生产构建

```bash
npx tauri build
```

---

## 发布新版本

```bash
git tag v0.3.0
git push origin main
git push origin v0.3.0
```

GitHub Actions 会自动构建并创建 Release。

---

## 许可

MIT License
