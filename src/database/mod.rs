pub mod models;
pub mod schema;

use anyhow::{anyhow, Result};
use diesel::{
    connection::SimpleConnection,
    r2d2::{ConnectionManager, Pool},
    BelongingToDsl, Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use models::{Attachment, Macro, NewAttachment, NewMacro};
use schema::{attachment, macro_};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone)]
pub struct Database {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    pub fn connect(url: impl AsRef<str>) -> Result<Database> {
        let manager = ConnectionManager::<SqliteConnection>::new(url.as_ref());
        let pool = Pool::builder().build(manager)?;

        let conn = &mut pool.get()?;

        conn.batch_execute("PRAGMA foreign_keys = ON")?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|why| anyhow!("Migration failed: {why}"))?;

        Ok(Self { pool })
    }

    pub fn get_macros(&self) -> Result<Vec<Macro>> {
        let mut conn = self.pool.get()?;

        Ok(macro_::table.select(Macro::as_select()).load(&mut conn)?)
    }

    pub fn get_macro(&self, name: impl AsRef<str>) -> Result<Option<(Macro, Vec<Attachment>)>> {
        let mut conn = self.pool.get()?;

        let r#macro = match macro_::table
            .filter(macro_::name.eq(name.as_ref()))
            .select(Macro::as_select())
            .get_result(&mut conn)
            .optional()?
        {
            Some(r#macro) => r#macro,
            None => return Ok(None),
        };

        let attachments = Attachment::belonging_to(&r#macro)
            .select(Attachment::as_select())
            .load(&mut conn)?;

        Ok(Some((r#macro, attachments)))
    }

    pub fn create_macro(
        &self,
        name: String,
        description: String,
        content: String,
        attachments: Vec<String>,
    ) -> Result<()> {
        let mut conn = self.pool.get()?;

        conn.transaction::<(), diesel::result::Error, _>(|conn| {
            let r#macro = diesel::insert_into(macro_::table)
                .values(&NewMacro {
                    name: &name,
                    description: &description,
                    content: &content,
                })
                .on_conflict(macro_::name)
                .do_update()
                .set((
                    macro_::description.eq(&description),
                    macro_::content.eq(&content),
                ))
                .returning(Macro::as_returning())
                .get_result(conn)?;

            // In the case of an update, delete old attachments
            diesel::delete(attachment::table)
                .filter(attachment::macro_id.eq(r#macro.id))
                .execute(conn)?;

            let attachments = attachments
                .iter()
                .map(|link| NewAttachment {
                    link,
                    macro_id: r#macro.id,
                })
                .collect::<Vec<_>>();

            diesel::insert_into(attachment::table)
                .values(&attachments)
                .execute(conn)?;

            Ok(())
        })?;

        Ok(())
    }

    pub fn delete_macro(&self, name: &str) -> Result<bool> {
        let mut conn = self.pool.get()?;

        let result =
            diesel::delete(macro_::dsl::macro_.filter(macro_::name.eq(name))).execute(&mut conn)?;

        Ok(result > 0)
    }
}
