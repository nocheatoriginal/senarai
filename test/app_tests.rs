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
            watched_episodes: 0,
        },
        Entry {
            id: Uuid::parse_str("772d2d49-9ce7-4db7-bd33-8dfb93617af4").unwrap(),
            title: "Test Entry 2".to_string(),
            season: 2,
            episode: 5,
            status: Status::Completed,
            watched_episodes: 0,
        },
        Entry {
            id: Uuid::parse_str("2cd6538f-944b-429e-b840-98ec89ed49ef").unwrap(),
            title: "Test Entry 3".to_string(),
            season: 1,
            episode: 0,
            status: Status::Planning,
            watched_episodes: 0,
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
    let entries = vec![
        Entry {
            id: Uuid::new_v4(),
            title: "P1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Planning,
            watched_episodes: 0,
        },
        Entry {
            id: Uuid::new_v4(),
            title: "P2".to_string(),
            season: 1,
            episode: 1,
            status: Status::Planning,
            watched_episodes: 0,
        },
        Entry {
            id: Uuid::new_v4(),
            title: "W1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Watching,
            watched_episodes: 0,
        },
        Entry {
            id: Uuid::new_v4(),
            title: "C1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Completed,
            watched_episodes: 0,
        },
    ];
    let config = Config {
        storage_path: "dummy_path".to_string(),
    };
    let mut app = App::new(entries, config);

    // Initial state: [P1, P2, W1, C1]
    // App::new sets selected_index to the first planning entry, which is 0.

    // Move W1 (index 2) to Planning
    app.selected_index = 2;
    app.move_to(Status::Planning);

    // Expected state: [P1, P2, W1(P), C1]
    // The moved entry "W1" should now be at index 2 with status Planning.
    // The selected index should be updated to the new position.
    assert_eq!(app.selected_index, 2);
    assert_eq!(app.entry[2].title, "W1");
    assert_eq!(app.entry[2].status, Status::Planning);
    assert_eq!(app.entry[0].title, "P1");
    assert_eq!(app.entry[1].title, "P2");
    assert_eq!(app.entry[3].title, "C1");

    // Now, move P1 (index 0) to Completed
    app.selected_index = 0;
    app.move_to(Status::Completed);

    // Initial state for this move: [P1, P2, W1(P), C1]
    // After removing P1: [P2, W1(P), C1]
    // The last non-dropped item is C1 at index 2.
    // So, P1(C) is inserted at index 3.
    // Expected state: [P2, W1(P), C1, P1(C)]
    assert_eq!(app.selected_index, 3);
    assert_eq!(app.entry[3].title, "P1");
    assert_eq!(app.entry[3].status, Status::Completed);
    assert_eq!(app.entry[0].title, "P2");
    assert_eq!(app.entry[1].title, "W1");
    assert_eq!(app.entry[2].title, "C1");
}

#[test]
fn test_edit_entry_title() {
    let mut app = create_dummy_app();
    app.selected_index = 0;
    let new_title = "Edited Title".to_string();
    app.entry[app.selected_index].title = new_title.clone();
    assert_eq!(app.entry[0].title, new_title);
}