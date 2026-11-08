#[tokio::test]
async fn test_state_creation_and_atomic_write() {
    let tmp_dir = std::env::temp_dir();
    let test_file = tmp_dir.join(format!("test_schema_{}.dbml", std::process::id()));

    // Limpeza prévia
    let _ = tokio::fs::remove_file(&test_file).await;

    // Criação do estado e do arquivo inicial
    let state = dbml_cli::state::AppState::new(test_file.clone()).await.unwrap();

    let initial = state.content.read().await.clone();
    assert!(initial.contains("Table users"));

    // Gravação atômica de nova versão
    let updated = format!("{initial}\nTable logs {{ id int }}\n");
    state.save_content(updated.clone()).await.unwrap();

    let on_disk = tokio::fs::read_to_string(&test_file).await.unwrap();
    assert_eq!(on_disk, updated);

    // Gravação de layout
    let layout_json = r#"{"tables":{"users":{"x":100,"y":100}}}"#.to_string();
    state.save_layout(layout_json.clone()).await.unwrap();

    let layout_on_disk = tokio::fs::read_to_string(&state.layout_path).await.unwrap();
    assert_eq!(layout_on_disk, layout_json);

    // Limpeza
    let _ = tokio::fs::remove_file(&test_file).await;
    let _ = tokio::fs::remove_file(&state.layout_path).await;
}

#[tokio::test]
async fn test_hash_calculation_and_suppression() {
    let text1 = "Table users { id int }";
    let text2 = "Table users { id int }";
    let text3 = "Table users { id bigint }";

    assert_eq!(dbml_cli::state::compute_hash(text1), dbml_cli::state::compute_hash(text2));
    assert_ne!(dbml_cli::state::compute_hash(text1), dbml_cli::state::compute_hash(text3));
}
