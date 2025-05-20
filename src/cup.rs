use crate::qr::generate_qr_code;
use askama::Template;
use cot::admin::AdminModel;
use cot::auth::db::DatabaseUser;
use cot::db::{model, query, Auto, ForeignKey};
use cot::form::{Form};
use cot::html::Html;
use cot::request::extractors::{Path, RequestDb};
use cot::router::Urls;
use cot::Error;

pub async fn get_cup(
    urls: Urls,
    RequestDb(db): RequestDb,
    Path(id): Path<i32>,
) -> cot::Result<Html> {
    #[derive(Debug, Template)]
    #[template(path = "get_cup.html")]
    struct GetCupTemplate<'a> {
        urls: &'a Urls,
        cup: &'a Cup,
        owner: &'a DatabaseUser,
        qr: &'a str,
    }

    let mut cup = query!(Cup, $id == id)
        .get(&db)
        .await?
        .ok_or(Error::custom("Cup not found"))?;
    let owner = cup.owner.get(&db).await?.clone();
    let qr = generate_qr_code(cup.id.unwrap().to_string().as_bytes()).unwrap();

    let template = GetCupTemplate {
        urls: &urls,
        cup: &cup,
        owner: &owner,
        qr: &qr,
    };

    Ok(Html::new(template.render()?))
}

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
