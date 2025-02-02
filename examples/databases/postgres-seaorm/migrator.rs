use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(MigrationCreateTodos)]
    }
}
struct MigrationCreateTodos;
impl MigrationName for MigrationCreateTodos {
    fn name(&self) -> &str {
        "m_20231106_000001_create_todos_table"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for MigrationCreateTodos {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Todos::Storage)
                    .col(ColumnDef::new(Todos::Uuid).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Todos::Task).string().not_null())
                    .col(ColumnDef::new(Todos::Done).boolean().not_null())
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Todos::Storage).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Todos {
    Storage,
    Uuid,
    Task,
    Done,
}
