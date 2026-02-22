use foundationdb::api::FdbApiBuilder;
use foundationdb::directory::Directory;
use foundationdb::tuple::pack;

const LEVELS: &[&str] = &[
    "intro",
    "for dummies",
    "remedial",
    "101",
    "201",
    "301",
    "mastery",
    "lab",
    "seminar",
];

const TYPES: &[&str] = &[
    "chem", "bio", "cs", "geometry", "calc", "alg", "film", "music", "art", "dance",
];

const TIMES: &[&str] = &[
    "2:00", "3:00", "4:00", "5:00", "6:00", "7:00", "8:00", "9:00", "10:00", "11:00", "12:00",
    "13:00", "14:00", "15:00", "16:00", "17:00", "18:00", "19:00",
];

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

    // Add classes to the class subspace
    let class_subspace = scheduling
        .subspace(&"classes")
        .expect("should get class subspace");
    trx.clear_subspace_range(&class_subspace);
    for level in LEVELS {
        for subject in TYPES {
            for time in TIMES {
                let class_name = format!("{time} {subject} {level}");
                trx.set(&class_subspace.pack(&class_name), &pack(&100_i64));
            }
        }
    }
    trx.commit().await?;

    Ok(())
}
