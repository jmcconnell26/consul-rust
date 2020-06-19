extern crate consul;
use consul::agent::Agent;

use consul::{Client, Config};

#[test]
fn agent_checks_test() {
    let client = set_up();

    let checks = client.checks();

    assert!(checks.is_ok());
}

#[test]
fn agent_members_test() {
    let client = set_up();
    let members = client.members(false).unwrap();
    assert_eq!(members.len(), 3);
}

#[test]
fn agent_reload_test() {
    let client = set_up();

    client.reload();
}

fn set_up() -> Client {
    let config = Config::new().unwrap();
    let client = Client::new(config);

    client
}
