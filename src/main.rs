mod spine;
mod corn_kernel;
mod cortex;
mod crypto;
mod eku;
mod merkle;
mod security;
mod protocol;
mod collective;
mod vm;
mod tui;

use spine::Spine;
use cortex::Cortex;
use std::time::Duration;
use tokio::time::sleep;
use std::env;

use eku::{Eku, EkuHeader, EkuType};
use corn_kernel::Z8Saturator;
use crypto::HopeKeyPair;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Entrópia validáció — VM / konténer környezetben kritikus
    if let Err(e) = security::entropy_check() {
        eprintln!("[SECURITY WARNING] {}", e);
        // Nem fatális — loggoljuk és folytatjuk, de jelezzük
    }

    let args: Vec<String> = env::args().collect();
    
    if args.iter().any(|arg| arg == "--agent") {
        run_agent().await?;
    } else if args.iter().any(|arg| arg == "--tui") {
        let mut app = tui::TuiApp::new()?;
        app.run()?;
    } else {
        let msg = if let Some(idx) = args.iter().position(|arg| arg == "--msg") {
            args.get(idx + 1).map(|s| s.as_str()).unwrap_or("Szia!")
        } else {
            "Szia Rongyasz! Sorold fel a legfontosabb eszkozeidet!"
        };
        run_kernel(msg).await?;
    }
    
    Ok(())
}

async fn run_agent() -> Result<(), Box<dyn std::error::Error>> {
    println!("════════════════════════════════════════════════════");
    println!("  RONGYÁSZ AGENT v2.0  —  Autonomous Cortex init");
    println!("════════════════════════════════════════════════════");

    let spine_id: [u8; 16] = *b"AGENT-CORTEX-Z8\x00";
    let mut spine = Spine::open_default(spine_id)?;
    println!("[AGENT] Spine connected: {}", spine.path);

    let cortex = Cortex::new("abab6.5s-chat");
    println!("[AGENT] Cortex ready (Model: MiniMax M2.5)");

    let mut vm = vm::HopeVM::new();
    println!("[AGENT] HOPE-VM initialized (Isolated Sandbox)");

    let mut last_processed_seq = spine.writer_seq();
    println!("[AGENT] Initial writer_seq: {}", last_processed_seq);

    println!("════════════════════════════════════════════════════");
    println!("  AGENT ACTIVE — Monitoring Spine...");
    println!("════════════════════════════════════════════════════");

    loop {
        let current_w = spine.writer_seq();

        if current_w > last_processed_seq {
            let r_seq = spine.reader_seq();
            println!("[AGENT] New activity: last={} current={} reader={}", last_processed_seq, current_w, r_seq);
            let target_seq = last_processed_seq;
            if let Some(kernel) = spine.read(target_seq) {
                let full_data = kernel.deep_read();
                
                let msg = std::str::from_utf8(full_data)
                    .unwrap_or("")
                    .trim_matches(char::from(0))
                    .trim();

                if !msg.is_empty() && !msg.starts_with("AGENT:") {
                    println!("[AGENT] Message detected: '{}'", msg);
                    println!("[AGENT] Thinking with MiniMax...");
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                    
                    let start = std::time::Instant::now();
                    match cortex.think(msg).await {
                        Ok(reply) => {
                            let duration = start.elapsed();
                            println!("[AGENT] Response received in {:?}", duration);
                            println!("[REPLY] {}", reply);

                            let mut response_kernel = corn_kernel::CornKernel::empty();
                            let agent_reply = format!("AGENT: {}", reply);
                            
                            // Válasz beírása több rétegbe ha kell
                            let bytes = agent_reply.as_bytes();
                            for (i, chunk) in bytes.chunks(32).enumerate().take(8) {
                                response_kernel.write_layer(i, chunk);
                            }
                            spine.write(&response_kernel);
                        }
                        Err(e) => println!("[AGENT] API Error: {:?}", e),
                    }
                }
            }
            last_processed_seq = target_seq + 1;
        }
        sleep(Duration::from_millis(500)).await;
    }
}

async fn run_kernel(main_msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("════════════════════════════════════════════════════");
    println!("  HOPE-OS SPINE  —  Silent Hope Protocol boot");
    println!("════════════════════════════════════════════════════\n");

    let keypair = HopeKeyPair::generate();
    let spine_id: [u8; 16] = *b"HOPE-STRATOS-Z8\x00";
    let mut spine = Spine::open_default(spine_id)?;

    println!("── Z8 Saturation ──────────────────────────────────");
    let mut saturator = Z8Saturator::new(0xD0E5);
    
    // Üzenet darabolása rétegekre
    for chunk in main_msg.as_bytes().chunks(32) {
        saturator.saturate(chunk);
    }

    println!("\n── Spine write & EKU signing ──────────────────────");
    let seq0 = spine.write(&saturator.kernel);
    println!("Spine wrote seq={seq0}");

    // EKU generálás (Máté kódja alapján)
    let sender_id: [u8; 16] = *b"INSTANCE-A-Z8-01";
    let mem_chain_ref: [u8; 16] = *b"HOPE-CHAIN-V1-00";
    let payload = saturator.kernel.flatten();
    
    let header = EkuHeader::new(
        EkuType::Execute, 
        0x01, 
        sender_id, 
        seq0 + 1, 
        mem_chain_ref, 
        payload.len() as u32
    );
    
    let mut eku = Eku::new(header, payload);
    keypair.sign(&mut eku);
    
    println!("[KERNEL] EKU signed: {:x?}", &eku.signature[..8]);
    
    println!("════════════════════════════════════════════════════");
    println!("  HOPE-OS SPINE mission accomplished.");
    println!("════════════════════════════════════════════════════");
    Ok(())
}
