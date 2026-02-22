use foundationdb::api::FdbApiBuilder;
use foundationdb::directory::Directory;
use foundationdb::tuple::Subspace;
use foundationdb::tuple::{pack, unpack};
use foundationdb::Database;
use foundationdb::RangeOption;

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
    let attends_subspace = scheduling
        .subspace(&"attends")
        .expect("should get class subspace");
    trx.clear_subspace_range(&class_subspace);
    trx.clear_subspace_range(&attends_subspace);
    for level in LEVELS {
        for subject in TYPES {
            for time in TIMES {
                let class_name = format!("{time} {subject} {level}");
                trx.set(&class_subspace.pack(&class_name), &pack(&100_i64));
            }
        }
    }
    trx.commit().await?;

    let available_classes = get_available_classes(&db).await;

    Ok(())
}

async fn get_available_classes(db: &Database) -> Vec<String> {
    let trx = db.create_trx().expect("could not start transaction");
    let directory = foundationdb::directory::DirectoryLayer::default();
    let path = vec![String::from("scheduling")];
    let scheduling = directory
        .create_or_open(&trx, &path, None, None)
        .await
        .expect("failed to create directory");
    let class_subspace = scheduling
        .subspace(&"classes")
        .expect("should get class subspace");

    let range = RangeOption::from(&class_subspace);

    let got_range = trx
        .get_range(&range, 1024, false)
        .await
        .expect("should get range of values");
    let mut available_classes = Vec::<String>::new();

    for key_value in got_range.iter() {
        let count: i64 = unpack(key_value.value()).expect("failed to decode count");

        if count > 0 {
            let class_name: String = class_subspace
                .unpack(key_value.key())
                .expect("Failed to get class name");
            available_classes.push(class_name);
        }
    }

    available_classes
}

async fn signup(db: &Database, student: &str, class_name: &str) -> foundationdb::FdbResult<()> {
    let trx = db.create_trx().expect("could not start transaction");
    let directory = foundationdb::directory::DirectoryLayer::default();
    let path = vec![String::from("scheduling")];
    let scheduling = directory
        .create_or_open(&trx, &path, None, None)
        .await
        .expect("failed to create directory");
    let class_subspace = scheduling
        .subspace(&"classes")
        .expect("should get class subspace");
    let attends_subspace = scheduling
        .subspace(&"attends")
        .expect("should get class subspace");

    let attends_key = attends_subspace.pack(&(student, class_name));
    if trx
        .get(&attends_key, true)
        .await
        .expect("should get attends")
        .is_some()
    {
        return Ok(());
    }

    let class_key = class_subspace.pack(&class_name);
    let available_seats: i64 = unpack(
        &trx.get(&class_key, true)
            .await
            .expect("get failed")
            .expect("class seats not initialized"),
    )
    .expect("failed to unpack");

    if available_seats <= 0 {
        panic!("No available seats");
    }

    let student_subspace = attends_subspace.subspace(&student);
    let attends_range = RangeOption::from(&student_subspace);
    if trx
        .get_range(&attends_range, 1024, false)
        .await
        .expect("should get range")
        .len()
        >= 5
    {
        panic!("student already has taken max classes");
    }

    trx.set(&class_key, &pack(&(available_seats - 1)));
    trx.set(&attends_key, &pack(&""));
    trx.commit().await.expect("commit should complete");

    Ok(())
}
