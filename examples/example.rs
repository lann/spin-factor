use outbound_http_factor::OutboundHttpFactor;
use spin_factors::SpinFactors;
use wasi_factor::WasiFactor;

#[derive(SpinFactors)]
pub struct MyFactors {
    wasi: WasiFactor,
    outbound_http: OutboundHttpFactor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::component::Linker::new(&engine);
        MyFactors::add_to_linker(&mut linker).unwrap();
        let factors = MyFactors::new(WasiFactor, OutboundHttpFactor);
        let data = factors.build_data().unwrap();
        let mut store = wasmtime::Store::new(&engine, data);
        let component = wasmtime::component::Component::new(&engine, b"(component)").unwrap();
        let _instance = linker.instantiate(&mut store, &component).unwrap();
    }
}

fn main() {}
