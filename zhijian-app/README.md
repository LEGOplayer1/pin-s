# 纸间 · 便利贴 Windows 桌面应用

> 基于 Electron 构建的「精致极简主义」便利贴应用。
> 设计灵感：莫兰迪色系 + 霞鹜文楷字体 + 拟物纸张纹理。

---

## ✨ 功能特性

- **独立窗口**：每张便利贴是一个独立的无边框窗口，可自由放置、自由缩放
- **6 种莫兰迪色**：米白 / 雾蓝 / 淡粉 / 草绿 / 薰衣草 / 鹅黄 — 每张便利贴可随时切换颜色
- **置顶 & 穿透**：一键置顶（始终显示），可选鼠标穿透模式
- **Markdown 支持**：标题、粗体、斜体、代码、引用
- **系统托盘**：右键托盘菜单可新建 / 显示 / 隐藏 / 退出
- **单实例**：再次启动仅新建便利贴而不是重复启动
- **快捷键**：
  - `Ctrl + N`：新建便利贴
  - `Ctrl + W`：关闭当前便利贴

---

## 📁 项目结构

```
zhijian-app/
├── package.json          # 依赖与构建配置
├── main.js               # Electron 主进程（窗口管理 / IPC / 托盘）
├── preload.js            # 预加载脚本（暴露安全 API 给渲染层）
├── renderer/
│   └── index.html        # 单张便利贴 UI（HTML + CSS + JS 一体）
├── .gitignore
└── README.md
```

---

## 🚀 快速开始

### 环境要求

- Node.js **18+**
- Windows 10 / 11 （推荐）

### 1. 安装依赖

```bash
cd zhijian-app
npm install
```

### 2. 开发运行

```bash
npm start
```

启动后你会看到桌面上出现一张欢迎便利贴。点击右上角 `+` 可新建更多便利贴，每张便利贴可独立拖拽、缩放、切换颜色。

### 3. 打包为 Windows 应用

```bash
# 仅打包不签名（快速测试用）
npm run dist

# 生成便携版（单个 .exe，免安装）
npm run dist:portable
```

打包完成后，产物在 `dist/` 目录下，包含：
- `纸间便利贴 Setup 1.0.0.exe` — NSIS 安装包（带桌面快捷方式）
- `纸间便利贴 1.0.0 portable.exe` — 便携版，双击即用

> 💡 首次执行 `npm run dist` 时，`electron-builder` 会自动下载 Windows 构建所需的 `winCodeSign` 与 `nsis` 工具，若网络较慢可手动挂代理。

---

## 🎨 自定义与扩展

### 修改默认颜色

打开 `renderer/index.html`，修改 `:root` 中的 CSS 变量即可：

```css
--note-cream:    #FAF6EE;
--note-blue:     #E8EDF2;
--note-pink:     #F2E8E8;
/* ... */
```

### 修改默认字号 / 字体

在 `renderer/index.html` 顶部 `font-family` 处替换为你的字体名称；
或使用本地字体文件（放入 `renderer/assets/fonts/`，通过 `@font-face` 引入）。

### 增加便利贴默认尺寸

编辑 `main.js` 中 `createNoteWindow` 的 `winW / winH` 初始值。

### 持久化内容（推荐扩展）

`preload.js` 已预留 `zhijian.saveNote / loadNotes / removeNote`，使用 `localStorage` 作为本地存储；
若需要跨会话恢复，可将 `localStorage` 替换为 `fs.writeFileSync` 到 `app.getPath('userData')` 目录。

### 增加「提醒 / 时间到通知」

- 主进程：`ipcMain.handle('notify', ...)` + `new Notification({ title, body }).show()`
- 渲染层：日期选择器 → 记录时间 → 主进程 `setTimeout` 到点弹出通知

---

## 🛠️ 常见问题

**Q：打包失败，报错 "winCodeSign" 下载失败？**
A：设置 npm 代理或为 electron-builder 配置镜像：
```bash
set ELECTRON_BUILDER_CACHE=C:\path\to\cache
npm run dist
```

**Q：运行时窗口背景不是透明，而是黑色？**
A：在 Windows 11 或部分设备上，若显卡驱动不完整可能出现。请确保系统已安装最新显卡驱动，并在 `main.js` 中保留 `transparent: true`。

**Q：想让便利贴开机自启？**
A：在 `app.whenReady()` 内追加：
```js
app.setLoginItemSettings({
  openAtLogin: true,
  path: process.execPath
});
```

---

## 📜 许可证

MIT License — 可自由修改与分发。
