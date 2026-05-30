use wasmer::{Instance, Module, Store, imports, Value};

pub const WASM_MAX_PAGES: u32 = 256; // 16MB limit

/// WASI időzítő importok — side-channel attack vector, explicit tiltva
/// Egy rosszindulatú plugin ezeken keresztül mérhetné az aláírási időket (cache-timing, Spectre-jellegű)
const BLOCKED_WASI_IMPORTS: &[(&str, &str)] = &[
    ("wasi_snapshot_preview1", "clock_time_get"),
    ("wasi_snapshot_preview1", "clock_res_get"),
    ("wasi_unstable",          "clock_time_get"),
    ("wasi_unstable",          "clock_res_get"),
    ("env",                    "emscripten_get_now"),
    ("env",                    "__wasi_clock_time_get"),
];

pub struct HopeVM {
    store: Store,
}

impl HopeVM {
    pub fn new() -> Self {
        Self { store: Store::default() }
    }

    pub fn execute(
        &mut self,
        wasm_bytes: &[u8],
        func_name: &str,
        args: &[Value],
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let module = Module::new(&self.store, wasm_bytes)?;
        self.validate_imports(&module)?;

        let import_object = imports! {};
        let instance = Instance::new(&mut self.store, &module, &import_object)?;
        self.enforce_memory_limit(&instance)?;

        let func = instance.exports.get_function(func_name)?;
        Ok(func.call(&mut self.store, args)?.to_vec())
    }

    pub fn run_heartbeat(
        &mut self,
        wasm_bytes: &[u8],
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let results = self.execute(wasm_bytes, "tick", &[])?;
        if let Some(Value::I32(val)) = results.first() {
            Ok(*val as u32)
        } else {
            Ok(0)
        }
    }

    /// Teljes import validáció:
    /// - Memória import tiltva (nem tudjuk limitálni)
    /// - WASI időzítő importok tiltva (side-channel védelem)
    fn validate_imports(&self, module: &Module) -> Result<(), Box<dyn std::error::Error>> {
        for import in module.imports() {
            // Memória import blokk
            if import.ty().memory().is_some() {
                return Err(format!(
                    "HopeVM: memória import megtagadva ('{}')",
                    import.name()
                ).into());
            }

            // WASI timer blokk — oldalcsatornás támadás ellen
            let module_name = import.module();
            let field_name  = import.name();
            for &(blocked_mod, blocked_fn) in BLOCKED_WASI_IMPORTS {
                if module_name == blocked_mod && field_name == blocked_fn {
                    return Err(format!(
                        "HopeVM: WASI timer import megtagadva ({}.{}) — side-channel védelem",
                        blocked_mod, blocked_fn
                    ).into());
                }
            }
        }
        Ok(())
    }

    fn enforce_memory_limit(&mut self, instance: &Instance) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(memory) = instance.exports.get_memory("memory") {
            let pages = memory.view(&self.store).size().0;
            if pages > WASM_MAX_PAGES {
                return Err(format!(
                    "HopeVM: memória limit túllépve ({} pages > {} max = {}MB)",
                    pages, WASM_MAX_PAGES, WASM_MAX_PAGES * 64 / 1024
                ).into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmer::wat2wasm;

    fn wasm_with_timer_import() -> Vec<u8> {
        // Wasm modul ami clock_time_get-et importál
        wat2wasm(br#"
            (module
                (import "wasi_snapshot_preview1" "clock_time_get"
                    (func $clock (param i32 i64 i32) (result i32)))
                (func (export "tick") (result i32)
                    (i32.const 0)
                    (i64.const 0)
                    (i32.const 0)
                    (call $clock)
                )
            )
        "#).expect("WAT parse").to_vec()
    }

    fn wasm_simple_add() -> Vec<u8> {
        wat2wasm(br#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#).expect("WAT parse").to_vec()
    }

    #[test]
    fn timer_import_blocked() {
        let mut vm = HopeVM::new();
        let result = vm.execute(&wasm_with_timer_import(), "tick", &[]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("timer") || msg.contains("clock"), "Várható timer blokk üzenet: {}", msg);
    }

    #[test]
    fn clean_wasm_executes() {
        let mut vm = HopeVM::new();
        use wasmer::Value;
        let result = vm.execute(&wasm_simple_add(), "add", &[Value::I32(3), Value::I32(4)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0], Value::I32(7));
    }
}
