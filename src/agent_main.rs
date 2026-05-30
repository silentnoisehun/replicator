mod spine;
mod corn_kernel;
mod cortex;

use spine::Spine;
use cortex::Cortex;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("════════════════════════════════════════════════════");
    print!("  RONGYÁSZ AGENT v2.0  —  Autonomous Cortex init");
    println!("\n════════════════════════════════════════════════════");

    // 1. Csatlakozás a Spine-hoz
    let spine_id: [u8; 16] = *b"AGENT-CORTEX-Z8\x00";
    let mut spine = Spine::open_default(spine_id)?;
    println!("[AGENT] Spine connected: {}", spine.path);

    // 2. Cortex inicializálása (MiniMax M2.5)
    let cortex = Cortex::new("abab6.5s-chat");
    println!("[AGENT] Cortex ready (Model: MiniMax M2.5)");

    let mut last_processed_seq = spine.writer_seq().saturating_sub(1);

    println!("════════════════════════════════════════════════════");
    println!("  AGENT ACTIVE — Monitoring Spine...");
    println!("════════════════════════════════════════════════════");

    loop {
        let current_w = spine.writer_seq();

        if current_w > last_processed_seq {
            // Új kukoricaszem érkezett
            let target_seq = last_processed_seq;
            
            if let Some(kernel) = spine.read(target_seq) {
                // Layer 0 beolvasása (ez a parancs/szöveg)
                if let Some(layer_0) = kernel.read_layer(0) {
                    let msg = std::str::from_utf8(layer_0)?
                        .trim_matches(char::from(0))
                        .trim();

                    if !msg.is_empty() && !msg.starts_with("AGENT:") {
                        println!("\n[USER] {}", msg);
                        print!("[AGENT] Thinking...");
                        
                        // Ágens gondolkodik
                        match cortex.think(msg).await {
                            Ok(reply) => {
                                println!("\r[AGENT] Response ready.");
                                println!("[REPLY] {}", reply);

                                // Válasz visszaírása a Spine-ba egy új CornKernel-ben
                                let mut response_kernel = corn_kernel::CornKernel::empty();
                                let agent_reply = format!("AGENT: {}", reply);
                                response_kernel.write_layer(0, agent_reply.as_bytes());
                                spine.write(&response_kernel);
                            }
                            Err(e) => println!("\r[AGENT] Error: {}", e),
                        }
                    }
                }
            }
            last_processed_seq = target_seq + 1;
        }

        sleep(Duration::from_millis(500)).await;
    }
}
