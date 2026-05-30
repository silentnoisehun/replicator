const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
    onSpineData: (callback) => ipcRenderer.on('spine-data', (event, data) => callback(data)),
    toggleFullscreen: () => ipcRenderer.send('toggle-fullscreen')
});
