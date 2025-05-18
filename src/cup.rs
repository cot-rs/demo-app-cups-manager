use cot::auth::Auth;
use cot::auth::db::DatabaseUser;
use cot::db::{model, query, Auto, ForeignKey};
use cot::request::extractors::{Path, RequestDb, };
use cot::request::{Request, RequestExt};
use cot::response::{ Response};
use cot::reverse_redirect;

pub async fn get_cup(RequestDb(db): RequestDb, Path(id): Path<i32>) -> cot::Result<String> {
    let cup = query!(Cup, $id == id).get(&db)
        .await?
        .map(|cup| cup.to_string())
        .unwrap_or_else(|| "Model not found".to_string());
    Ok(cup)
}

pub async fn create_cup(mut request: Request) -> cot::Result<Response> {
    let auth: Auth = request.extract_parts().await?;
    if !auth.user().is_authenticated() {
        //TODO: fix that redirect
        return Ok(reverse_redirect!(request, "login")?);
    }
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)] //, Form, AdminModel)]
#[model]
struct Cup {
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
