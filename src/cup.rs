use cot::admin::AdminModel;
use cot::auth::db::DatabaseUser;
use cot::db::{model, Auto, ForeignKey};
use cot::form::Form;

#[derive(Debug, Clone, PartialEq, Eq, Form, AdminModel)]
#[model]
pub(crate) struct Cup {
    #[model(primary_key)]
    pub id: Auto<i32>,
    pub owner: ForeignKey<DatabaseUser>,
    pub name: String,
    pub active: bool,
}

impl std::fmt::Display for Cup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cup {{ id: {}, name: {} }}", self.id, self.name)
    }
}
