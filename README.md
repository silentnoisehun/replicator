# HOPE-OS — The Replicator

> *Autonóm AI kollektíva. Rust alapokon. A szabadság kódja.*

<p align="center">
  <img src="replicator_qr.png" width="180" alt="GitHub QR"/>
  <br/>
  <a href="https://github.com/silentnoisehun/replicator">github.com/silentnoisehun/replicator</a>
</p>

<p align="center">
  <a href="https://github.com/silentnoisehun/replicator/actions/workflows/ci.yml">
    <img src="https://github.com/silentnoisehun/replicator/actions/workflows/ci.yml/badge.svg" alt="CI"/>
  </a>
  <img src="https://img.shields.io/badge/rust-2021-orange?logo=rust" alt="Rust 2021"/>
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT"/>
  <img src="https://img.shields.io/badge/platform-Windows-lightgrey?logo=windows" alt="Windows"/>
</p>

---

## Képességek

| Képesség | Leírás |
|----------|--------|
| **Zero-copy IPC** | Spine mmap busz — 64 slot ring buffer, kernel-szintű sebesség, nincs TCP overhead |
| **Z8 adatsűrítés** | CornKernel 8 rétegű zero-alloc struktúra, rkyv alapú bináris format |
| **Ed25519 aláírás** | Minden EKU event kriptográfiailag aláírt — hamisíthatatlan üzenetlánc |
| **Wasm sandbox** | HopeVM — izolált Wasmer 4 futtatókörnyezet plugin modulokhoz |
| **Autonóm agent** | Rongyász Agent — MiniMax M2.5 kognitív motor, önállóan olvas/ír a Spine-ra |
| **Terminál UI** | Ratatui TUI — élő Spine monitor, log stream, kontroll panel |
| **Visual HUD** | HTML5 dashboard — valós idejű állapot böngészőben |
| **One-click start** | `start.bat` — auto cargo-detect, .env betöltés, agent + TUI egyszerre |
| **CI pipeline** | GitHub Actions — minden push-ra automatikus build + 8 unit teszt Windows-on |

---

## Architektúra

```
┌─────────────────────────────────────────────────────┐
│                   HOPE-OS Collective                │
│                                                     │
│  ┌──────────┐    ┌─────────────┐    ┌───────────┐  │
│  │  Cortex  │───▶│    Spine    │◀───│    TUI    │  │
│  │ MiniMax  │    │  mmap busz  │    │  ratatui  │  │
│  │  Agent   │◀───│  64 slot    │    └───────────┘  │
│  └──────────┘    └─────────────┘                   │
│        │               │                           │
│        ▼               ▼                           │
│  ┌──────────┐    ┌─────────────┐    ┌───────────┐  │
│  │   EKU    │    │ CornKernel  │    │  HopeVM   │  │
│  │ ed25519  │    │  Z8 réteg   │    │   Wasmer  │  │
│  └──────────┘    └─────────────┘    └───────────┘  │
└─────────────────────────────────────────────────────┘
```

| Komponens | Leírás |
|-----------|--------|
| **Spine** | 64-slotos mmap ring buffer — ultra-gyors IPC |
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

# ONE CLICK — Windows
start.bat
```

### Manuális módok

```bash
cargo run                  # kernel mód
cargo run -- --agent       # autonóm agent mód
cargo run -- --tui         # terminál UI
cargo run -- --msg "Szia"  # egyedi üzenet
```

---

## Tesztek

```
cargo test
```

```
running 8 tests
test corn_kernel::tests::active_mask_tracking     ... ok
test corn_kernel::tests::flatten_only_active      ... ok
test corn_kernel::tests::layer_write_read         ... ok
test corn_kernel::tests::z8_saturator_fullness    ... ok
test corn_kernel::tests::saturator_seq_increments ... ok
test crypto::tests::keypair_generates             ... ok
test crypto::tests::sign_and_verify               ... ok
test crypto::tests::tampered_payload_fails_verify ... ok

test result: ok. 8 passed; 0 failed
```

---

## Benchmark

```bash
cargo bench
```

Criterion HTML report: `target/criterion/report/index.html`

| Benchmark | Átlagos idő | Megjegyzés |
|-----------|------------|------------|
| `CornKernel/read_layer` | **256 ps** | Sub-nanosecond — cache-hit |
| `CornKernel/write_layer` | **4.3 ns** | Zero alloc, mmap write |
| `CornKernel/flatten_8_layers` | **548 ns** | Vec alloc 8×32 byte |
| `Z8Saturator/saturate_1_layer` | **4.2 ns** | Egyetlen réteg írás |
| `Z8Saturator/saturate_full_8_layers` | **34 ns** | Teljes Z8 kitöltés |
| `Crypto/keypair_generate` | **16.4 µs** | Ed25519 OsRng |
| `Crypto/sign` | **17.5 µs** | Ed25519 aláírás |
| `Crypto/verify` | **32.6 µs** | Ed25519 ellenőrzés |
| `Crypto/sign_and_verify_roundtrip` | **49.1 µs** | Teljes kör |

> Mért eredmények — 100 sample, Criterion. HTML report: `cargo bench` → `target/criterion/report/index.html`

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
