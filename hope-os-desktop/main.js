const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');

function createWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    backgroundColor: '#020304',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  win.loadFile('index.html');

  const spinePath = 'C:\\Users\\mater\\.gemini\\tmp\\hope_spine.bin';
  if (fs.existsSync(spinePath)) {
    fs.watch(spinePath, () => {
      fs.readFile(spinePath, (err, data) => {
        if (!err) win.webContents.send('spine-data', data);
      });
    });
  }

  ipcMain.on('toggle-fullscreen', () => {
    win.setFullScreen(!win.isFullScreen());
  });
}

app.whenReady().then(createWindow);
app.on('window-all-closed', () => app.quit());
