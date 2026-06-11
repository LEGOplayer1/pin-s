const { app, BrowserWindow, ipcMain, Tray, Menu, screen, nativeImage } = require('electron');
const path = require('path');
const fs = require('fs');

const isDev = !app.isPackaged;
const isWindows = process.platform === 'win32';

let noteWindows = [];
let tray = null;

// ============================================================
// 创建单个便利贴窗口
// ============================================================
function createNoteWindow(options = {}) {
  const { x, y, color, size, content } = options;
  const winW = size?.w || 260;
  const winH = size?.h || 240;

  const display = screen.getPrimaryDisplay();
  const { width: screenW, height: screenH } = display.workAreaSize;

  const win = new BrowserWindow({
    width: winW,
    height: winH,
    x: x ?? Math.max(40, Math.floor(Math.random() * (screenW - winW - 80)) + 40),
    y: y ?? Math.max(40, Math.floor(Math.random() * (screenH - winH - 120)) + 40),
    minWidth: 180,
    minHeight: 140,
    frame: false,
    transparent: true,
    resizable: true,
    alwaysOnTop: false,
    skipTaskbar: true, // 便利贴窗口不显示在任务栏
    backgroundColor: '#00000000',
    hasShadow: true,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false
    }
  });

  win.loadFile(path.join(__dirname, 'renderer', 'index.html'), {
    query: {
      color: color || 'cream',
      content: content ? encodeURIComponent(content) : ''
    }
  });

  win.once('ready-to-show', () => {
    win.show();
    win.focus();
  });

  win.on('closed', () => {
    noteWindows = noteWindows.filter(w => w !== win);
  });

  noteWindows.push(win);
  return win;
}

// ============================================================
// IPC：便利贴窗口 -> 主进程 的系统操作
// ============================================================
ipcMain.handle('window:new', (evt, opts) => {
  createNoteWindow(opts || {});
  return true;
});

ipcMain.handle('window:close', (evt) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) win.close();
  return true;
});

ipcMain.handle('window:pin', (evt, pinned) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) win.setAlwaysOnTop(pinned, 'screen-saver');
  return true;
});

ipcMain.handle('window:minimize', (evt) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) {
    if (win.isMinimized()) win.restore();
    else win.minimize();
  }
  return true;
});

ipcMain.handle('window:setClickThrough', (evt, flag) => {
  // Windows 下可通过 setIgnoreMouseEvents 实现"鼠标穿透"
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) {
    if (flag) {
      win.setIgnoreMouseEvents(true, { forward: true });
    } else {
      win.setIgnoreMouseEvents(false);
    }
  }
  return true;
});

ipcMain.handle('window:drag', (evt, { dx, dy }) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (!win) return;
  const [x, y] = win.getPosition();
  win.setPosition(x + dx, y + dy);
});

ipcMain.handle('window:setPosition', (evt, { x, y }) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) win.setPosition(x, y);
});

ipcMain.handle('window:getPosition', (evt) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (!win) return null;
  const [x, y] = win.getPosition();
  return { x, y };
});

ipcMain.handle('window:setSize', (evt, { w, h }) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) win.setSize(w, h);
});

ipcMain.handle('window:getSize', (evt) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (!win) return null;
  const [w, h] = win.getSize();
  return { w, h };
});

ipcMain.handle('window:focus', (evt) => {
  const win = BrowserWindow.fromWebContents(evt.sender);
  if (win) {
    win.show();
    win.focus();
  }
});

ipcMain.handle('app:showAll', () => {
  noteWindows.forEach(w => {
    if (w.isMinimized()) w.restore();
    w.show();
    w.focus();
  });
  return true;
});

// ============================================================
// 系统托盘
// ============================================================
function createTray() {
  // 使用内联 SVG 生成托盘图标（避免外部二进制依赖）
  const trayIconPath = path.join(__dirname, 'renderer', 'assets', 'tray-icon.png');
  let image;
  try {
    image = nativeImage.createFromPath(trayIconPath);
    if (image.isEmpty()) throw new Error('icon not found');
  } catch (e) {
    // 回退：使用空图标
    image = nativeImage.createEmpty();
  }

  tray = new Tray(image);
  tray.setToolTip('纸间便利贴');

  const menu = Menu.buildFromTemplate([
    { label: '新建便利贴', click: () => createNoteWindow() },
    { type: 'separator' },
    { label: '显示全部', click: () => noteWindows.forEach(w => { if (w.isMinimized()) w.restore(); w.show(); }) },
    { label: '最小化全部', click: () => noteWindows.forEach(w => w.minimize()) },
    { type: 'separator' },
    { label: '退出 纸间', click: () => { app.isQuiting = true; app.quit(); } }
  ]);

  tray.setContextMenu(menu);

  tray.on('click', () => {
    if (noteWindows.length === 0) {
      createNoteWindow();
    } else {
      const hasVisible = noteWindows.some(w => w.isVisible() && !w.isMinimized());
      if (hasVisible) {
        noteWindows.forEach(w => w.minimize());
      } else {
        noteWindows.forEach(w => { w.restore(); w.show(); });
      }
    }
  });
}

// ============================================================
// 应用生命周期
// ============================================================
app.whenReady().then(() => {
  // Windows 下保持单实例
  const gotLock = app.requestSingleInstanceLock();
  if (!gotLock) {
    app.quit();
    return;
  }

  app.on('second-instance', () => {
    // 再次启动时，只是新建一张便利贴
    createNoteWindow();
  });

  // 默认启动时创建一张欢迎便利贴
  createNoteWindow({ color: 'cream' });

  if (isWindows) {
    try { createTray(); } catch (e) {
      console.warn('托盘初始化失败（图标资源缺失），继续运行...', e.message);
    }
  }

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createNoteWindow();
  });
});

app.on('window-all-closed', (e) => {
  // 便利贴全部关闭时，在 Windows 上保留托盘 + 应用运行，以便随时新建
  if (process.platform !== 'darwin') {
    // 不退出，托盘仍可用
  }
});
