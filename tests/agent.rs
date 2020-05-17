extern crate consul;
use consul::agent::Agent;
use consul::session::{Session, SessionEntry};
use consul::{Client, Config};

extern crate rand;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[test]
fn agent_checks_test() {
    let client = set_up();

    let checks = client.checks();

    assert!(checks.is_ok());
}

#[test]
fn agent_maintenance_mode_test() {
    let client = set_up();

    let initial_check_count = client.checks().unwrap().len();
    client.maintenance_mode(true, None).unwrap();

    let checks = client.checks().unwrap();
    assert_eq!(checks.len(), 1);

    client.maintenance_mode(false, None).unwrap();

    let checks = client.checks().unwrap();
    assert_eq!(checks.len(), initial_check_count);
}

#[test]
fn agent_members_test() {
    let client = set_up();

    let members = client.members(false).unwrap();

    assert_eq!(members.len(), 1);

    let member = members.iter().next().unwrap();
    let node_name = get_node_name_of_session(&client);

    assert_eq!(member.Name, node_name);
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

fn get_node_name_of_session(client: &Client) -> String {
    let unique_test_identifier: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

    let entry = SessionEntry {
        Name: Some(unique_test_identifier.to_string()),
        ..Default::default()
    };

    let (created_session_entry, _) = client.create(&entry, None).unwrap();
    let created_session_entry_id = created_session_entry.ID.unwrap();

    let (session_entries_info, _) = client.info(&created_session_entry_id, None).unwrap();
    let session_entry_info = session_entries_info.iter().next().unwrap();

    client.destroy(&created_session_entry_id, None).unwrap();

    String::from(session_entry_info.Node.as_ref().unwrap())
}
