use std::collections::HashMap;

use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};

#[derive(Clone, GraphQLObject)]
pub struct User {
    id: i32,
    name: String,
}

#[derive(Default, Clone)]
pub struct Database {
    ///this could be a database connection
    users: HashMap<i32, User>,
}
impl Database {
    pub fn new() -> Database {
        let mut users = HashMap::new();
        users.insert(
            1,
            User {
                id: 1,
                name: "Aron".to_string(),
            },
        );
        users.insert(
            2,
            User {
                id: 2,
                name: "Bea".to_string(),
            },
        );
        users.insert(
            3,
            User {
                id: 3,
                name: "Carl".to_string(),
            },
        );
        users.insert(
            4,
            User {
                id: 4,
                name: "Dora".to_string(),
            },
        );
        Database { users }
    }
    pub fn get_user(&self, id: &i32) -> Option<&User> {
        self.users.get(id)
    }
}

// To make our Database usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Database {}

// Queries represent the callable funcitons
pub struct Query;
#[graphql_object(context = Database)]
impl Query {
    fn apiVersion() -> String {
        "1.0".to_string()
    }
    #[graphql(arguments(id(description = "id of the user")))]
    fn user(database: &Database, id: i32) -> Option<&User> {
        database.get_user(&id)
    }
}

pub type Schema = RootNode<'static, Query, EmptyMutation<Database>, EmptySubscription<Database>>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Database>::new(),
        EmptySubscription::<Database>::new(),
    )
}
