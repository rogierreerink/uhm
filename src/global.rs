use deadpool::managed::Pool;
use deadpool_postgres::Manager;

pub struct AppState {
    pub db_pool: Pool<Manager>,
}
