use outbound_http_factor::OutboundHttpFactor;
use outbound_networking_factor::OutboundNetworkingFactor;
use spin_factors::SpinFactors;
use wasi_factor::WasiFactor;

#[derive(SpinFactors)]
pub struct MyFactors {
    wasi: WasiFactor,
    outbound_networking_factor: OutboundNetworkingFactor,
    outbound_http_factor: OutboundHttpFactor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let engine = wasmtime::Engine::default();
        let mut linker = wasmtime::component::Linker::new(&engine);
        MyFactors::init(&mut linker).unwrap();

        let factors = MyFactors {
            wasi: WasiFactor,
            outbound_networking_factor: OutboundNetworkingFactor,
            outbound_http_factor: OutboundHttpFactor,
        };
        let data = factors.build_data().unwrap();

        let mut store = wasmtime::Store::new(&engine, data);
        let component = wasmtime::component::Component::new(&engine, b"(component)").unwrap();
        let _instance = linker.instantiate(&mut store, &component).unwrap();
    }
}

fn main() {}
