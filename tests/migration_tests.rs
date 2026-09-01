#[tokio::test]
#[ignore = "Requires active PostgreSQL test instance"]
async fn test_migrations_execute_cleanly() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL needed for test");
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    
    let migrator = sqlx::migrate!("./migrations");
    migrator.run(&pool).await.expect("Migration failed to apply cleanly");
}