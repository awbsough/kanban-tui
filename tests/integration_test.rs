use kanban_tui::Board;

#[test]
fn test_complete_workflow() {
    // Create a new board
    let mut board = Board::new("Project Board".to_string());

    // Add tasks to the "To Do" column
    let task1_id = board.add_task(0, "Implement feature X".to_string()).unwrap();
    let task2_id = board.add_task(0, "Write documentation".to_string()).unwrap();
    let task3_id = board.add_task(0, "Add tests".to_string()).unwrap();

    // Verify tasks were added
    assert_eq!(board.columns[0].tasks.len(), 3);
    assert_eq!(board.columns[1].tasks.len(), 0);
    assert_eq!(board.columns[2].tasks.len(), 0);

    // Move task1 to "In Progress"
    board.move_task(0, 1, task1_id).unwrap();
    assert_eq!(board.columns[0].tasks.len(), 2);
    assert_eq!(board.columns[1].tasks.len(), 1);

    // Move task2 to "In Progress"
    board.move_task(0, 1, task2_id).unwrap();
    assert_eq!(board.columns[0].tasks.len(), 1);
    assert_eq!(board.columns[1].tasks.len(), 2);

    // Complete task1 (move to "Done")
    board.move_task(1, 2, task1_id).unwrap();
    assert_eq!(board.columns[1].tasks.len(), 1);
    assert_eq!(board.columns[2].tasks.len(), 1);

    // Verify the completed task
    assert_eq!(board.columns[2].tasks[0].id, task1_id);
    assert_eq!(board.columns[2].tasks[0].title, "Implement feature X");

    // Task3 should still be in "To Do"
    assert_eq!(board.columns[0].tasks[0].id, task3_id);
}

#[test]
fn test_custom_board() {
    let column_names = vec![
        "Backlog".to_string(),
        "Ready".to_string(),
        "In Progress".to_string(),
        "Review".to_string(),
        "Done".to_string(),
    ];

    let mut board = Board::with_columns("Custom Board".to_string(), column_names);

    assert_eq!(board.columns.len(), 5);
    assert_eq!(board.columns[0].name, "Backlog");
    assert_eq!(board.columns[3].name, "Review");

    // Add and move tasks through the custom workflow
    let task_id = board.add_task(0, "New feature".to_string()).unwrap();
    board.move_task(0, 1, task_id).unwrap(); // Backlog -> Ready
    board.move_task(1, 2, task_id).unwrap(); // Ready -> In Progress
    board.move_task(2, 3, task_id).unwrap(); // In Progress -> Review
    board.move_task(3, 4, task_id).unwrap(); // Review -> Done

    assert_eq!(board.columns[4].tasks.len(), 1);
    assert_eq!(board.columns[4].tasks[0].title, "New feature");
}

#[test]
fn test_serialization() {
    let mut board = Board::new("Test Board".to_string());
    board.add_task(0, "Task 1".to_string()).unwrap();
    board.add_task(1, "Task 2".to_string()).unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&board).unwrap();

    // Deserialize back
    let deserialized: Board = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "Test Board");
    assert_eq!(deserialized.columns.len(), 3);
    assert_eq!(deserialized.columns[0].tasks[0].title, "Task 1");
    assert_eq!(deserialized.columns[1].tasks[0].title, "Task 2");
}

#[test]
fn test_error_handling() {
    let mut board = Board::new("Test".to_string());

    // Try to add task to non-existent column
    let result = board.add_task(10, "Task".to_string());
    assert!(result.is_err());

    // Try to move task from non-existent column
    let task_id = board.add_task(0, "Task".to_string()).unwrap();
    let result = board.move_task(10, 1, task_id);
    assert!(result.is_err());

    // Try to move non-existent task
    let result = board.move_task(0, 1, 9999);
    assert!(result.is_err());
}
