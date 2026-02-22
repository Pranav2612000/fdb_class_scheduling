use foundationdb::api::FdbApiBuilder;
use foundationdb::directory::Directory;
use foundationdb::directory::DirectoryOutput::DirectorySubspace;

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
    class_scheduling_example()
        .await
        .expect("should complete the test");

    drop(network);
    println!("Test fdb example completed");
}

async fn class_scheduling_example() -> foundationdb::FdbResult<()> {
    let db = foundationdb::Database::default()?;

    // create a new directory
    let trx = db.create_trx()?;
    let directory = foundationdb::directory::DirectoryLayer::default();
    let path = vec![String::from("scheduling")];
    let scheduling = directory
        .create_or_open(&trx, &path, None, None)
        .await
        .expect("failed to create directory");
    let scheduling = match scheduling {
        DirectorySubspace(d) => d,
        _ => panic!("did not create a subspace"),
    };
    trx.commit().await?;

    Ok(())
}
