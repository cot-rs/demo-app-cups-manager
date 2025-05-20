use crate::qr::generate_qr_code;
use askama::Template;
use cot::admin::AdminModel;
use cot::auth::db::DatabaseUser;
use cot::auth::Auth;
use cot::db::{model, query, Auto, Database, ForeignKey, Model};
use cot::form::{Form, FormContext, FormResult};
use cot::html::Html;
use cot::request::extractors::{Path, RequestDb, RequestForm};
use cot::response::{IntoResponse, Response};
use cot::router::Urls;
use cot::{reverse_redirect, Error};

pub async fn create_cup_page(urls: Urls, auth: Auth) -> cot::Result<Response> {
    #[derive(Debug, Template)]
    #[template(path = "create_cup.html")]
    struct CreateCupTemplate<'a> {
        urls: &'a Urls,
        form: <CreateCupForm as Form>::Context,
    }

    if !auth.user().is_authenticated() {
        return Ok(reverse_redirect!(urls, "cot_admin:login")?);
    }

    let template = CreateCupTemplate {
        urls: &urls,
        form: <<CreateCupForm as Form>::Context as FormContext>::new(),
    };

    Html::new(template.render()?).into_response()
}

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

pub async fn create_cup_form(
    urls: Urls,
    auth: Auth,
    RequestDb(db): RequestDb,
    RequestForm(input): RequestForm<CreateCupForm>,
) -> cot::Result<Response> {
    if !auth.user().is_authenticated() {
        return Ok(reverse_redirect!(urls, "cot_admin:login")?);
    }
    let cup = match input {
        FormResult::Ok(data) => {
            create_cup_impl(&db, auth.user().id().unwrap().as_int().unwrap(), data.name).await?
        }
        FormResult::ValidationError(_) => todo!("show errors in frontend"),
    };

    Ok(reverse_redirect!(urls, "get-cup", id = cup.id.unwrap())?)
}

#[derive(Debug, Form)]
pub struct CreateCupForm {
    name: String,
}

async fn create_cup_impl(db: &Database, owner_id: i64, name: String) -> cot::Result<Cup> {
    let owner = query!(DatabaseUser, $id == owner_id).get(db).await?;
    let Some(owner) = owner else {
        return Err(Error::custom("Owner not found"));
    };

    let mut cup = Cup {
        id: Auto::default(),
        owner: ForeignKey::from(owner),
        name,
        active: false,
    };
    cup.insert(db).await?;

    Ok(cup)
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
