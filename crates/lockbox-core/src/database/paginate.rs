use serde::{Serialize, Deserialize};

use crate::database::{
    orm::{ConnectionTrait, EntityTrait, PaginatorTrait, Select},
    errors::{DatabaseError, DatabaseResult},
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub count: u64,
    pub next_page: Option<u64>,
    pub previous_page: Option<u64>,
}


impl<T> Default for Page<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            count: 0,
            next_page: None,
            previous_page: None,
        }
    }
}


pub async fn paginate<C, E>(
    database: &C,
    query: Select<E>,
    page: u64,
    per_page: u64,
) -> DatabaseResult<Page<E::Model>>
where
    C: ConnectionTrait + Send + Sync,
    E: EntityTrait,
    <E as EntityTrait>::Model: Send + Sync + 'static,
{
    let paginator = query.paginate(database, per_page);
    // Sea-ORM paginator is 0-indexed
    let items = paginator.fetch_page(page-1).await.map_err(DatabaseError::from)?;

    let count = paginator.num_items().await.map_err(DatabaseError::from)?;
    let next_page = if items.len() < per_page as usize { None } else { Some(page + 1) };
    let previous_page = if page > 1 { Some(page - 1) } else { None };

    Ok(Page {
        items,
        count,
        next_page,
        previous_page,
    })
}