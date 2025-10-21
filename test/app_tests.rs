use senarai::app::App;
use senarai::config::Config;
use senarai::{Entry, Status};
use uuid::Uuid;

fn create_dummy_app() -> App {
    let entries = vec![
        Entry {
            id: Uuid::parse_str("4b974928-1aa0-4596-be76-7427b4a4e343").unwrap(),
            title: "Test Entry 1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Watching,
        },
        Entry {
            id: Uuid::parse_str("772d2d49-9ce7-4db7-bd33-8dfb93617af4").unwrap(),
            title: "Test Entry 2".to_string(),
            season: 2,
            episode: 5,
            status: Status::Completed,
        },
        Entry {
            id: Uuid::parse_str("2cd6538f-944b-429e-b840-98ec89ed49ef").unwrap(),
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
