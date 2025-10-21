use senarai::app::App;
use senarai::config::Config;
use senarai::{Entry, Status};

fn create_dummy_app() -> App {
    let entries = vec![
        Entry {
            id: 0,
            title: "Test Entry 1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Watching,
        },
        Entry {
            id: 1,
            title: "Test Entry 2".to_string(),
            season: 2,
            episode: 5,
            status: Status::Completed,
        },
        Entry {
            id: 2,
            title: "Test Entry 3".to_string(),
            season: 1,
            episode: 0,
            status: Status::Planning,
        },
    ];
    let config = Config {
        storage_path: "dummy_path".to_string(),
    };
    App::new(entries, config)
}

#[test]
fn test_add_entry() {
    let mut app = create_dummy_app();
    let initial_len = app.entry.len();
    app.add_entry("New Test Entry".to_string());
    assert_eq!(app.entry.len(), initial_len + 1);
    assert_eq!(app.entry.last().unwrap().title, "New Test Entry");
    assert_eq!(app.entry.last().unwrap().status, Status::Planning);
    assert_eq!(app.entry.last().unwrap().season, 1);
    assert_eq!(app.entry.last().unwrap().episode, 0);
}

#[test]
fn test_remove_entry() {
    let mut app = create_dummy_app();
    let initial_len = app.entry.len();
    app.selected_index = 0;
    app.remove_entry();
    assert_eq!(app.entry.len(), initial_len - 1);
    assert_eq!(app.entry[0].title, "Test Entry 2");
}

#[test]
fn test_next_episode() {
    let mut app = create_dummy_app();
    app.selected_index = 0;
    app.next_episode();
    assert_eq!(app.entry[0].episode, 2);

    app.selected_index = 2;
    app.next_episode();
    assert_eq!(app.entry[2].episode, 1);
}

#[test]
fn test_prev_episode() {
    let mut app = create_dummy_app();
    app.selected_index = 0;
    app.prev_episode();
    assert_eq!(app.entry[0].episode, 0);

    app.prev_episode();
    assert_eq!(app.entry[0].episode, 0);
    assert_eq!(app.entry[0].season, 1);

    app.entry[0].season = 2;
    app.entry[0].episode = 0;
    app.prev_episode();
    assert_eq!(app.entry[0].season, 1);
    assert_eq!(app.entry[0].episode, 0);
}

#[test]
fn test_next_season() {
    let mut app = create_dummy_app();
    app.selected_index = 0;
    app.next_season();
    assert_eq!(app.entry[0].season, 2);
    assert_eq!(app.entry[0].episode, 0);
}

#[test]
fn test_move_to() {
    let mut app = create_dummy_app();
    app.selected_index = 0;
    app.move_to(Status::Completed);
    assert_eq!(app.entry[0].status, Status::Completed);

    app.move_to(Status::Planning);
    assert_eq!(app.entry[0].status, Status::Planning);
}

#[test]
fn test_edit_entry_title() {
    let mut app = create_dummy_app();
    app.selected_index = 0;
    let new_title = "Edited Title".to_string();
    app.entry[app.selected_index].title = new_title.clone();
    assert_eq!(app.entry[0].title, new_title);
}
