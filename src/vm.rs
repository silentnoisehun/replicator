use wasmer::{Instance, Module, Store, imports, Value};

pub struct HopeVM {
    store: Store,
}

impl HopeVM {
    pub fn new() -> Self {
        Self {
            store: Store::default(),
        }
    }

    /// Futtat egy Wasm modult bájtokból
    pub fn execute(&mut self, wasm_bytes: &[u8], func_name: &str, args: &[Value]) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let module = Module::new(&self.store, wasm_bytes)?;
        
        // Alapértelmezett importok (pl. konzol log, ha kell)
        let import_object = imports! {};
        
        let instance = Instance::new(&mut self.store, &module, &import_object)?;
        let func = instance.exports.get_function(func_name)?;
        
        let result = func.call(&mut self.store, args)?;
        Ok(result.to_vec())
    }

    /// ORA/Rongyász speciális 'Heartbeat' moduljainak futtatása
    pub fn run_heartbeat(&mut self, wasm_bytes: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
        let results = self.execute(wasm_bytes, "tick", &[])?;
        if let Some(Value::I32(val)) = results.first() {
            Ok(*val as u32)
        } else {
            Ok(0)
        }
    }
}
