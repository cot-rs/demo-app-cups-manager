use crate::qr::{generate_qr_code, scan_qr_code};
use askama::Template;
use cot::admin::AdminModel;
use cot::auth::db::DatabaseUser;
use cot::auth::Auth;
use cot::db::{model, query, Auto, Database, ForeignKey, Model};
use cot::form::fields::InMemoryUploadedFile;
use cot::form::{Form, FormContext, FormResult};
use cot::html::Html;
use cot::json::Json;
use cot::request::extractors::{Path, RequestDb, RequestForm};
use cot::response::{IntoResponse, Response};
use cot::router::Urls;
use cot::{reverse_redirect, Error, StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

pub async fn create_cup(
    RequestDb(db): RequestDb,
    Json(input): Json<CreateCupApiForm>,
) -> Json<CreateCupApiResponse> {
    let mut cup = create_cup_impl(&db, input.owner, input.name).await.unwrap();

    Json(CreateCupApiResponse {
        id: cup.id.unwrap(),
        owner: cup.owner.get(&db).await.unwrap().username().to_owned(),
        name: cup.name,
        active: cup.active,
    })
}

#[derive(Debug, Form, JsonSchema, Deserialize)]
pub struct CreateCupApiForm {
    owner: i64,
    name: String,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct CreateCupApiResponse {
    pub id: i32,
    pub owner: String,
    pub name: String,
    pub active: bool,
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

pub async fn scan_cup_page(urls: Urls) -> cot::Result<Html> {
    #[derive(Debug, Template)]
    #[template(path = "scan_cup.html")]
    struct ScanCupTemplate<'a> {
        urls: &'a Urls,
        form: <ScanCupForm as Form>::Context,
    }

    let template = ScanCupTemplate {
        urls: &urls,
        form: <<ScanCupForm as Form>::Context as FormContext>::new(),
    };

    Ok(Html::new(template.render()?))
}

#[derive(Debug, Form)]
pub struct ScanCupForm {
    file: InMemoryUploadedFile,
}

pub async fn scan_cup_form(
    urls: Urls,
    RequestDb(db): RequestDb,
    RequestForm(input): RequestForm<ScanCupForm>,
) -> cot::Result<Response> {
    let data = input.unwrap().file.content().clone();

    let scanned = scan_qr_code(data).map_err(|e| Error::custom(e.to_string()))?;
    let id = i32::from_str(&scanned).map_err(|e| Error::custom(e.to_string()))?;

    let cup = query!(Cup, $id == id).get(&db).await?;

    let Some(cup) = cup else {
        return "Cup not found"
            .with_status(StatusCode::NOT_FOUND)
            .into_response();
    };

    Ok(reverse_redirect!(urls, "get-cup", id = cup.id.unwrap())?)
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
