extern crate consul;
use consul::health::Health;
use consul::{Client, Config};

extern crate rand;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[test]
fn health_service_test_passing_only() {
    let client = set_up();

    let (service_entries, query_meta) = client
        .service("consul", Option::None, true, Option::None)
        .unwrap();

    assert_eq!(service_entries.len(), 3);

    let service_entry = service_entries.iter().next().unwrap();

    assert_eq!(service_entry.Service.Service, "consul");
    assert!(query_meta.last_index.unwrap() > 0, "index must be positive");
}

#[test]
fn health_service_test_non_existant() {
    let client = set_up();

    let non_existant_service_name: String =
        thread_rng().sample_iter(&Alphanumeric).take(16).collect();

    let (service_entries, meta_query) = client
        .service(&non_existant_service_name, Option::None, true, Option::None)
        .unwrap();

    assert_eq!(service_entries.len(), 0);
    assert!(meta_query.last_index.unwrap() > 0, "index must be positive");
}

fn set_up() -> Client {
    let config = Config::new().unwrap();
    let client = Client::new(config);

    return client;
}
