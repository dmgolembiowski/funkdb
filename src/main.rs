use rusqlite::{self, Connection};
struct App {
    store: Store,
}

#[derive(Clone)]
struct Card {
    id: u32,
}

impl App {
    pub(crate) fn new(store: Store) -> Self {
        Self { store }
    }

    fn query_card(&self, needle: Card) -> Option<Card> {
        self.store.query_card(needle)
    }

    fn card_exists(&self, card: Card) -> bool {
        self.query_card(card).is_some()
    }
}

struct Store {
    db: Connection,
}

impl Store {
    fn new() -> Self {
        let db = rusqlite::Connection::open_in_memory().expect("disk access");
        &db.execute("CREATE TABLE Card ( id INTEGER PRIMARY KEY);", ());
        &db.execute("INSERT INTO Card (id) values (10);", ()); // values (
        Store { db }
    }

    fn query_card(&self, card: Card) -> Option<Card> {
        let query = format!("select * from Card where id = {} limit 1;", card.clone().id);
        let mut stmt = self.db.prepare(query.as_str()).unwrap();
        let res = stmt.query([]);
        match res {
            Ok(mut rows) => {
                while let Ok(Some(row)) = rows.next() {
                    let id: u32 = row.get(0).unwrap();
                    return Some(Card { id });
                }
                None
            }
            Err(_e) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_first_card() {
        let store = Store::new();
        let app = App::new(store);
        let card = Card { id: 10 };
        assert!(&app.card_exists(card));
    }

    #[test]
    fn add_real_card() {
        // Create the DB
        let db = Connection::open_in_memory().expect("...");

        // create a card
        let card = Card { id: 10 };

        let mut store = Store::new();

        // create an app using that store

        let app = App::new(store);
        let queried = &app.query_card(card.clone());
        assert!(queried.is_some());
        assert!(&app.card_exists(card));
    }
}
use anyhow::{Error, Result};
fn main() -> Result<()> {
    Ok(())
}
