use crate::qr::{generate_qr_code, scan_qr_code};
use cot::auth::db::DatabaseUser;
use cot::auth::{Auth, UserId};
use cot::db::{model, query, Auto, Database, ForeignKey, Model};
use cot::form::{Form, FormResult};
use cot::json::Json;
use cot::request::extractors::{Path, RequestDb, RequestForm};
use cot::request::Request;
use cot::response::{IntoResponse, Response};
use cot::router::Urls;
use cot::{reverse_redirect, Error, StatusCode};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use schemars::JsonSchema;

pub async fn get_cup(RequestDb(db): RequestDb, Path(id): Path<i32>) -> cot::Result<String> {
    let cup = query!(Cup, $id == id)
        .get(&db)
        .await?
        .map(|cup| cup.to_string())
        .unwrap_or_else(|| "Model not found".to_string());
    Ok(cup)
}

#[derive(Deserialize, Form)]
pub struct NewCup {
    pub name: String,
}

pub async fn create_cup(
    urls: Urls,
    auth: Auth,
    RequestDb(db): RequestDb,
    Json(input): Json<NewCup>,
) -> cot::Result<Response> {
    if !auth.user().is_authenticated() {
        //TODO: fix that redirect
        return Ok(reverse_redirect!(urls, "login")?);
    }
    create_cup_impl(db, auth.user().id().unwrap(), input).await
}

pub async fn create_cup_form(
    urls: Urls,
    auth: Auth,
    RequestDb(db): RequestDb,
    RequestForm(input): RequestForm<NewCup>,
) -> cot::Result<Response> {
    if !auth.user().is_authenticated() {
        //TODO: fix that redirect
        return Ok(reverse_redirect!(urls, "login")?);
    }
    match input {
        FormResult::Ok(data) => create_cup_impl(db, auth.user().id().unwrap(), data).await,
        FormResult::ValidationError(e) => todo!("show errors in frontend"),
    }
    //TODO: show successful path
}

pub async fn get_cup_qr(RequestDb(db): RequestDb, Path(id): Path<i32>) -> cot::Result<Response> {
    let cup = query!(Cup, $id == id).get(&db).await?;

    let Some(cup) = cup else {
        return "Model not found"
            .with_status(StatusCode::NOT_FOUND)
            .into_response();
    };

    generate_qr_code(cup.id.unwrap().to_string().as_bytes())
        .map_err(|err| Error::custom(format!("{:?}", err)))
        .into_response()
}

pub async fn scan_cup_qr(RequestDb(db): RequestDb, request: Request) -> cot::Result<Response> {
    let data = request.into_body().into_bytes().await?;

    let scanned = scan_qr_code(data).map_err(|e| Error::custom(e.to_string()))?;
    let id = i32::from_str(&scanned).map_err(|e| Error::custom(e.to_string()))?;

    let cup = query!(Cup, $id == id).get(&db).await?;

    let Some(cup) = cup else {
        return "Cup not found"
            .with_status(StatusCode::NOT_FOUND)
            .into_response();
    };

    print!("{cup}").into_response()
}

async fn create_cup_impl(
    db: Arc<Database>,
    owner_id: UserId,
    data: NewCup,
) -> cot::Result<Response> {
    let owner = query!(DatabaseUser, $id == owner_id.as_int().unwrap())
        .get(&db)
        .await?;
    let Some(owner) = owner else {
        return "Owner not found"
            .with_status(StatusCode::UNAUTHORIZED)
            .into_response();
    };

    let mut cup = Cup {
        id: Auto::default(),
        owner: ForeignKey::from(owner),
        name: data.name,
        active: false,
    };
    cup.insert(&db).await?;

    print!("{cup}")
        .with_status(StatusCode::CREATED)
        .into_response()
}

#[derive(Debug, Clone, PartialEq, Eq)] //, Form, AdminModel)]
// TODO: implement deserialization for Auto and ForeignKey
// #[derive(Deserialize, JsonSchema)]
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
