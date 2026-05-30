import time
import os
import random

spine_path = r"C:\Users\mater\.gemini\tmp\hope_spine.bin"

print(f"[RONGYASZ] Artificial Pulse Active: {spine_path}")

while True:
    try:
        with open(spine_path, "wb") as f:
            # Generáljunk 256 bájtnyi "életet"
            data = bytearray([random.randint(0, 255) for _ in range(256)])
            f.write(data)
        time.sleep(0.1) # 10Hz-es pulzálás
    except Exception as e:
        print(f"Error: {e}")
        time.sleep(1)
