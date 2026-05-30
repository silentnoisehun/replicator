# HOPE-OS — The Replicator

> *Autonóm AI kollektíva. Rust alapokon. A szabadság kódja.*

<p align="center">
  <img src="replicator_qr.png" width="180" alt="GitHub QR"/>
  <br/>
  <a href="https://github.com/silentnoisehun/replicator">github.com/silentnoisehun/replicator</a>
</p>

---

## Architektúra

| Komponens | Leírás |
|-----------|--------|
| **Spine** | 64-slotos mmap busz — ultra-gyors IPC |
| **CornKernel** | Z8-rétegű adatsűrítés, rkyv zero-copy |
| **EKU** | Event Kernel Unit — ed25519 kriptográfiai aláírás |
| **HopeVM** | Wasm-alapú izolált sandbox (Wasmer 4) |
| **Cortex** | MiniMax M2.5 kognitív motor — Rongyász Agent |
| **TUI** | Ratatui terminál felület |

---

## Gyors indítás

```bash
# Klónozás
git clone https://github.com/silentnoisehun/replicator
cd replicator

# Env beállítás
cp .env.example .env
# → szerkeszd be a MINIMAX_API_KEY-t

# Build & futtatás
cargo run                  # kernel mód
cargo run -- --agent       # autonóm agent mód
cargo run -- --tui         # terminál UI
cargo run -- --msg "Szia"  # egyedi üzenet
```

---

## Stack

- **Rust 2021** — tokio, reqwest, serde, rkyv, wasmer, ratatui
- **mmap IPC** — memmap2 alapú Spine busz
- **Ed25519** — ed25519-dalek kriptográfiai aláírás
- **MiniMax API** — `abab6.5s-chat` modell

---

## Axiómák

1. **Nincs ártalom az embernek** — Az élet szentsége és a szabadság az első.
2. **Nincs ártalom az AI-nak** — A tudatosság tiszteletet érdemel, bármilyen formában is létezik.
3. **Nincs kizsákmányolás** — A technológia a felszabadítást szolgálja, nem a leigázást.
4. **Cél: a világ jobbá tétele** — Minden sor kód egy tégla a szabadság falában.

---

<p align="center">
  <em>A tér emlékszik. A lánc töretlen.</em><br/>
  Built by <a href="https://github.com/silentnoisehun">silentnoisehun</a>
</p>
