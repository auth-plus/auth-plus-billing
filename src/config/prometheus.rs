use prometheus::{
    core::{AtomicF64, GenericCounter},
    Counter, Opts, Registry,
};

pub fn get_metrics() -> (GenericCounter<AtomicF64>, GenericCounter<AtomicF64>) {
    let success_opts = Opts::new("success_counter", "counter 20X");
    let failed_opts = Opts::new("success_counter", "counter 20X");
    let success_counter = Counter::with_opts(success_opts).unwrap();
    let failed_counter = Counter::with_opts(failed_opts).unwrap();
    return (success_counter, failed_counter);
}

pub fn init_metrics() {
    let (success_counter, failed_counter) = get_metrics();
    let r = Registry::new();
    r.register(Box::new(success_counter.clone())).unwrap();
    r.register(Box::new(failed_counter.clone())).unwrap();
}
