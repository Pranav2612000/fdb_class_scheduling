use foundationdb::api::FdbApiBuilder;

#[tokio::main]
async fn main() {
    println!("Starting test fdb app");

    let fdb_builder = FdbApiBuilder::default();
    let fdb_builder = fdb_builder.set_runtime_version(730);

    let network_builder = fdb_builder
        .build()
        .expect("should be able to build from fdb builder");
    let network = unsafe { network_builder.boot() };

    // Perform fdb operations

    drop(network);
    println!("Test fdb example completed");
}
