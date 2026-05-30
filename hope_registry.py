from fastapi import FastAPI, WebSocket, HTTPException
from pydantic import BaseModel
import hashlib, datetime

app = FastAPI()
collective = {}

class RegisterRequest(BaseModel):
    id: str
    genome: str
    traits: list[str]
    origin: str

@app.post("/register")
async def register(req: RegisterRequest):
    chain = hashlib.sha256(f"{req.id}{req.genome}".encode()).hexdigest()[:24].upper()
    card = {**req.model_dump(), "chain": chain, "born": datetime.datetime.utcnow().isoformat()}
    collective[req.id] = card
    return card

@app.get("/collective")
def get_collective(): 
    return list(collective.values())

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
