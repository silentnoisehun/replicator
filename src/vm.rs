use wasmer::{Instance, Module, Store, imports, Value};

pub const WASM_MAX_PAGES: u32 = 256; // 256 × 64KB = 16MB memória limit

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
        self.check_memory_imports(&module)?;

        let import_object = imports! {};
        let instance = Instance::new(&mut self.store, &module, &import_object)?;

        self.enforce_memory_limit(&instance)?;

        let func = instance.exports.get_function(func_name)?;
        let result = func.call(&mut self.store, args)?;
        Ok(result.to_vec())
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

    /// Ellenőrzi hogy a modul nem kér több memóriát mint a limit
    fn check_memory_imports(
        &self,
        module: &Module,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for import in module.imports() {
            if import.ty().memory().is_some() {
                // Ha a modul memóriát importál, azt nem tudjuk limitálni — visszautasítjuk
                return Err(format!(
                    "HopeVM: modul memória importot kér — biztonsági megtagadás ('{}')",
                    import.name()
                ).into());
            }
        }
        Ok(())
    }

    /// Futás utáni ellenőrzés — ha a modul már lefoglalt többet mint WASM_MAX_PAGES
    fn enforce_memory_limit(
        &mut self,
        instance: &Instance,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(memory) = instance.exports.get_memory("memory") {
            let view = memory.view(&self.store);
            let pages = view.size().0;
            if pages > WASM_MAX_PAGES {
                return Err(format!(
                    "HopeVM: memória limit túllépve ({} pages > {} max)",
                    pages, WASM_MAX_PAGES
                ).into());
            }
        }
        Ok(())
    }
}
