const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('zhijian', {
  // 窗口操作
  newNote: (opts) => ipcRenderer.invoke('window:new', opts),
  closeNote: () => ipcRenderer.invoke('window:close'),
  pin: (pinned) => ipcRenderer.invoke('window:pin', pinned),
  minimize: () => ipcRenderer.invoke('window:minimize'),
  focus: () => ipcRenderer.invoke('window:focus'),
  setClickThrough: (flag) => ipcRenderer.invoke('window:setClickThrough', flag),

  // 位置/尺寸
  drag: (dx, dy) => ipcRenderer.invoke('window:drag', { dx, dy }),
  setPosition: (x, y) => ipcRenderer.invoke('window:setPosition', { x, y }),
  getPosition: () => ipcRenderer.invoke('window:getPosition'),
  setSize: (w, h) => ipcRenderer.invoke('window:setSize', { w, h }),
  getSize: () => ipcRenderer.invoke('window:getSize'),

  // 应用级
  showAll: () => ipcRenderer.invoke('app:showAll'),

  // 读取 URL 参数
  getParams: () => {
    const q = new URLSearchParams(window.location.search);
    return {
      color: q.get('color') || 'cream',
      content: q.get('content') ? decodeURIComponent(q.get('content')) : ''
    };
  },

  // 本地持久化（使用 localStorage）
  saveNote: (id, data) => {
    try {
      const all = JSON.parse(localStorage.getItem('zhijian_notes') || '{}');
      all[id] = data;
      localStorage.setItem('zhijian_notes', JSON.stringify(all));
    } catch (e) { /* ignore */ }
  },
  loadNotes: () => {
    try { return JSON.parse(localStorage.getItem('zhijian_notes') || '{}'); }
    catch (e) { return {}; }
  },
  removeNote: (id) => {
    try {
      const all = JSON.parse(localStorage.getItem('zhijian_notes') || '{}');
      delete all[id];
      localStorage.setItem('zhijian_notes', JSON.stringify(all));
    } catch (e) { /* ignore */ }
  }
});
