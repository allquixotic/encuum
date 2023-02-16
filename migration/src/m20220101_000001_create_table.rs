/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use entity::{prelude::*};
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
        tcs.push(Schema::create_table_from_entity(&schema, Applications));
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
            "Applications",
        ] {
            let mut t = Table::drop();
            t.table(Alias::new(i));
            manager.drop_table(t).await.ok();
        }

        Ok(())
    }
}
