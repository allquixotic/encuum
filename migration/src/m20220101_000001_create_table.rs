use entity::prelude::*;
use sea_orm_migration::{prelude::*, sea_orm::Schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let be = manager.get_database_backend();
        let schema = Schema::new(be);
        let mut tcs: Vec<TableCreateStatement> = vec![];
        tcs.push(Schema::create_table_from_entity(&schema, Subforums));
        tcs.push(Schema::create_table_from_entity(&schema, Images));
        tcs.push(Schema::create_table_from_entity(&schema, CategoryNames));
        tcs.push(Schema::create_table_from_entity(&schema, ForumPosts));
        tcs.push(Schema::create_table_from_entity(&schema, ForumPresets));
        tcs.push(Schema::create_table_from_entity(&schema, ForumThreads));
        for tc in tcs {
            manager.create_table(tc).await.unwrap();
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for i in vec![
            "Subforums",
            "Images",
            "CategoryNames",
            "ForumPosts",
            "ForumPresets",
            "ForumThreads",
        ] {
            let mut t = Table::drop();
            t.table(Alias::new(i));
            manager.drop_table(t).await.ok();
        }

        Ok(())
    }
}
