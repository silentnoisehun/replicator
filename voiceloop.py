import mmap
import os
import time
import struct
import threading
import asyncio
import websockets
import edge_tts
import requests
import json

SPINE_PATH = r"C:\Users\mater\.gemini\tmp\hope_spine.bin"
SPINE_SIZE = 2048
SLOT_SIZE = 64
FIELD_COUNT = 12
RING_START = 808
RING_CAPACITY = 19

class SpineBridge:
    def __init__(self, path):
        self.path = path
        # Győződjünk meg róla, hogy a mappa létezik
        os.makedirs(os.path.dirname(path), exist_ok=True)
        if not os.path.exists(path):
            with open(path, "wb") as f:
                f.write(b'\x00' * SPINE_SIZE)
        
        self.file = open(path, "r+b")
        self.mm = mmap.mmap(self.file.fileno(), SPINE_SIZE)

    def get_seqs(self):
        w = struct.unpack_from("<Q", self.mm, 0)[0]
        r = struct.unpack_from("<Q", self.mm, 8)[0]
        return w, r

    def push(self, data):
        w = struct.unpack_from("<Q", self.mm, 0)[0]
        slot_idx = w % RING_CAPACITY
        offset = RING_START + slot_idx * SLOT_SIZE
        raw = data[:SLOT_SIZE]
        self.mm[offset:offset+len(raw)] = raw
        if len(raw) < SLOT_SIZE:
            self.mm[offset+len(raw):offset+SLOT_SIZE] = b'\x00' * (SLOT_SIZE - len(raw))
        struct.pack_into("<Q", self.mm, 0, w + 1)

bridge = SpineBridge(SPINE_PATH)

async def speak(text):
    print(f"[VOICE] TTS: {text}")
    try:
        communicate = edge_tts.Communicate(text, "hu-HU-NoemiNeural")
        await communicate.save("temp_voice.mp3")
        os.system("start /min temp_voice.mp3")
    except Exception as e:
        print(f"[VOICE] TTS Error: {e}")

async def ollama_chat(prompt):
    url = "http://localhost:11434/api/generate"
    data = {
        "model": "phi:latest",
        "prompt": f"Te vagy Rongyász, egy bio-inspirált AI kódsebész. Válaszolj röviden, magyarul: {prompt}",
        "stream": True
    }
    full_response = ""
    try:
        with requests.post(url, json=data, stream=True) as response:
            for line in response.iter_lines():
                if line:
                    chunk = json.loads(line.decode('utf-8'))
                    token = chunk.get("response", "")
                    full_response += token
                    yield token
    except Exception as e:
        yield f"[Hiba: {e}]"
    
    # Ha kész, dobjuk a Spine-ba tts-hez
    bridge.push(f"TXT:{full_response}".encode('utf-8'))

async def voice_watcher():
    print("[VOICE] Watcher active.")
    _, last_r = bridge.get_seqs()
    while True:
        w, r = bridge.get_seqs()
        if w > last_r:
            offset = RING_START + (last_r % RING_CAPACITY) * SLOT_SIZE
            data = bridge.mm[offset:offset+SLOT_SIZE].rstrip(b'\x00')
            try:
                msg = data.decode('utf-8')
                if msg.startswith("TXT:"):
                    await speak(msg[4:])
            except: pass
            last_r += 1
        await asyncio.sleep(0.2)

async def ws_handler(websocket):
    print(f"[WS] Connected")
    try:
        async for message in websocket:
            data = bytearray(message)
            if not data: continue
            cmd = data[0]
            
            if cmd == 0x20: # MSG_CHAT
                prompt = data[2:].decode('utf-8')
                print(f"[USER] {prompt}")
                
                # Ollama streamelés vissza a UI-ra
                async for chunk in ollama_chat(prompt):
                    await websocket.send(bytearray([0x21]) + chunk.encode('utf-8'))
                
                await websocket.send(bytearray([0x22])) # END
            
            elif cmd == 0x10: # SYNC
                await websocket.send(bytearray([0x10]))

    except websockets.exceptions.ConnectionClosed:
        print("[WS] Closed")

async def main():
    server = websockets.serve(ws_handler, "0.0.0.0", 7701)
    await asyncio.gather(server, voice_watcher())

if __name__ == "__main__":
    asyncio.run(main())
