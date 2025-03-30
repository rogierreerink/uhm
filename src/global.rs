use crate::db::{Db, DbPostgres};

pub struct AppState {
    pub db: AppDb,
}

impl AppState {
    pub fn db(&self) -> &impl Db {
        match &self.db {
            AppDb::Postgres(db) => db,
        }
    }
}

pub enum AppDb {
    Postgres(DbPostgres),
}
