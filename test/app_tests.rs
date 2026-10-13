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
            max_episodes: 0,
        },
        Entry {
            id: Uuid::parse_str("772d2d49-9ce7-4db7-bd33-8dfb93617af4").unwrap(),
            title: "Test Entry 2".to_string(),
            season: 2,
            episode: 5,
            status: Status::Completed,
            watched_episodes: 0,
            max_episodes: 0,
        },
        Entry {
            id: Uuid::parse_str("2cd6538f-944b-429e-b840-98ec89ed49ef").unwrap(),
            title: "Test Entry 3".to_string(),
            season: 1,
            episode: 0,
            status: Status::Planning,
            watched_episodes: 0,
            max_episodes: 0,
        },
    ];
    let config = Config {
        storage_path: "dummy_path".to_string(),
        ..Config::default()
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
            max_episodes: 0,
        },
        Entry {
            id: Uuid::new_v4(),
            title: "P2".to_string(),
            season: 1,
            episode: 1,
            status: Status::Planning,
            watched_episodes: 0,
            max_episodes: 0,
        },
        Entry {
            id: Uuid::new_v4(),
            title: "W1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Watching,
            watched_episodes: 0,
            max_episodes: 0,
        },
        Entry {
            id: Uuid::new_v4(),
            title: "C1".to_string(),
            season: 1,
            episode: 1,
            status: Status::Completed,
            watched_episodes: 0,
            max_episodes: 0,
        },
    ];
    let config = Config {
        storage_path: "dummy_path".to_string(),
        ..Config::default()
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

#[test]
fn test_selected_entry_progress_rules() {
    let mut app = create_dummy_app();

    app.selected_index = 0;
    app.entry[0].watched_episodes = 3;
    app.entry[0].max_episodes = 10;
    app.entry[0].status = Status::Watching;
    assert_eq!(app.selected_entry_progress(), 30);

    app.entry[0].max_episodes = 0;
    assert_eq!(app.selected_entry_progress(), 0);

    app.entry[0].max_episodes = 10;
    app.entry[0].status = Status::Completed;
    assert_eq!(app.selected_entry_progress(), 100);

    app.entry[0].status = Status::Watching;
    app.entry[0].watched_episodes = 12;
    assert_eq!(app.selected_entry_progress(), 100);
}

struct DatabaseDirectory(std::path::PathBuf);

impl DatabaseDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("senarai-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn config(&self) -> Config {
        Config {
            storage_path: self.0.to_str().unwrap().to_owned(),
            ..Config::default()
        }
    }
}

impl Drop for DatabaseDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn database_switch_persists_and_keeps_writes_isolated() {
    use senarai::database;
    let directory = DatabaseDirectory::new();
    let original = directory.config();
    database::init_db(&original).unwrap();
    let mut app = App::new(Vec::new(), original.clone());
    app.add_entry("Original".into());
    let mut second = original.clone();
    second.database_file = "other.db".into();
    database::init_db(&second).unwrap();
    std::fs::write(directory.0.join("ignore.txt"), "").unwrap();
    std::fs::create_dir(directory.0.join("directory.db")).unwrap();

    app.open_database_picker().unwrap();
    assert_eq!(
        app.database_files,
        vec![std::path::PathBuf::from("other.db"), "senarai.db".into()]
    );
    assert_eq!(app.database_list_state.selected(), Some(1));
    app.database_list_state.select(Some(0));
    app.open_selected_database().unwrap();
    assert!(app.entry.is_empty());
    assert_eq!(app.selected_index, 0);
    app.add_entry("Other".into());
    app.next_episode();
    assert_eq!(
        database::load_entry(&original).unwrap()[0].title,
        "Original"
    );
    assert_eq!(database::load_entry(&original).unwrap()[0].episode, 0);
    assert_eq!(database::load_entry(&second).unwrap()[0].episode, 1);

    let mut restarted = directory.config();
    restarted.restore_last_database().unwrap();
    assert_eq!(restarted.database_file, second.database_file);
    assert_eq!(database::load_entry(&restarted).unwrap()[0].title, "Other");
    std::fs::remove_file(second.database_path()).unwrap();
    restarted.restore_last_database().unwrap();
    assert_eq!(restarted.database_file, original.database_file);
}

#[test]
fn failed_database_switch_preserves_current_entries_and_selection() {
    let directory = DatabaseDirectory::new();
    let config = directory.config();
    senarai::database::init_db(&config).unwrap();
    config.remember_database().unwrap();
    let mut app = App::new(Vec::new(), config);
    app.add_entry("Keep me".into());
    std::fs::write(directory.0.join("broken.db"), "not a SQLite database").unwrap();
    app.open_database_picker().unwrap();
    app.database_list_state.select(Some(0));
    assert!(app.open_selected_database().is_err());
    assert_eq!(app.entry[0].title, "Keep me");
    assert_eq!(
        app.config.database_file,
        std::path::PathBuf::from("senarai.db")
    );
    let mut restarted = directory.config();
    restarted.restore_last_database().unwrap();
    assert_eq!(restarted.database_file, app.config.database_file);

    std::fs::remove_file(directory.0.join("broken.db")).unwrap();
    assert!(app.open_selected_database().is_err());
    assert!(!directory.0.join("broken.db").exists());
}

#[test]
fn picker_renders_empty_and_scrolls_to_selected_database() {
    use ratatui::{backend::TestBackend, Terminal};
    let directory = DatabaseDirectory::new();
    let mut app = App::new(Vec::new(), directory.config());
    app.open_database_picker().unwrap();
    assert_eq!(app.database_list_state.selected(), None);
    app.open_selected_database().unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| senarai::ui::draw_ui(frame, &mut app))
        .unwrap();
    let content = |terminal: &Terminal<TestBackend>| {
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    };
    assert!(content(&terminal).contains("No .db files found."));
    for index in 0..40 {
        std::fs::write(directory.0.join(format!("{index:02}.db")), "").unwrap();
    }
    app.open_database_picker().unwrap();
    app.database_list_state.select(Some(39));
    terminal
        .draw(|frame| senarai::ui::draw_ui(frame, &mut app))
        .unwrap();
    assert!(content(&terminal).contains("> 39.db"));
}
