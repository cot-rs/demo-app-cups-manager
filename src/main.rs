mod cup;
mod migrations;
mod qr;

use crate::cup::{create_cup, create_cup_form, get_cup, get_cup_qr, scan_cup_qr};
use askama::Template;
use async_trait::async_trait;
use cot::admin::AdminApp;
use cot::auth::db::{DatabaseUser, DatabaseUserApp};
use cot::cli::CliMetadata;
use cot::common_types::Password;
use cot::db::migrations::SyncDynMigration;
use cot::html::Html;
use cot::middleware::{AuthMiddleware, LiveReloadMiddleware, SessionMiddleware};
use cot::project::{MiddlewareContext, RegisterAppsContext, RootHandlerBuilder};
use cot::router::method::{get, post};
use cot::router::{Route, Router};
use cot::static_files::{StaticFile, StaticFilesMiddleware};
use cot::{static_files, App, AppBuilder, BoxedHandler, Project, ProjectContext};
use cot::openapi::swagger_ui::SwaggerUi;
use cot::router::method::openapi::{api_get, api_post};

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> cot::Result<Html> {
    let index_template = IndexTemplate {};
    let rendered = index_template.render()?;

    Ok(Html::new(rendered))
}

struct DemoAppCupsManagerApp;

#[async_trait]
impl App for DemoAppCupsManagerApp {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }
    async fn init(&self, context: &mut ProjectContext) -> cot::Result<()> {
        // Check if admin user exists
        let admin_username = std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
        let user = DatabaseUser::get_by_username(context.database(), &*admin_username).await?;
        if user.is_none() {
            let password =
                std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "password".to_string());
            // Create admin user
            DatabaseUser::create_user(
                context.database(),
                &admin_username,
                &Password::new(&password),
            )
            .await?;
        }
        Ok(())
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("/", get(index), "index"),
            Route::with_handler_and_name("/cup/{id}", get(get_cup), "get-cup"),
            // TODO: figure out cup deserialization first
            // Route::with_api_handler_and_name("/cup/{id}", api_get(get_cup), "get-cup"),
            Route::with_api_handler_and_name("/cup/{id}/qr", api_get(get_cup_qr), "qr-cup"),
            Route::with_handler_and_name("/cup", post(create_cup), "create-cup"),
            // TODO: figure out cup deserialization first
            // Route::with_api_handler_and_name("/cup", api_post(create_cup), "create-cup"),
            Route::with_handler_and_name("/cup/form", post(create_cup_form), "create-cup-form"),
            Route::with_api_handler_and_name("/cup/scan", api_post(scan_cup_qr), "scan-cup"),
        ])
    }

    fn migrations(&self) -> Vec<Box<SyncDynMigration>> {
        cot::db::migrations::wrap_migrations(migrations::MIGRATIONS)
    }

    fn static_files(&self) -> Vec<StaticFile> {
        static_files!("css/main.css")
    }
}

struct DemoAppCupsManagerProject;

impl Project for DemoAppCupsManagerProject {
    fn cli_metadata(&self) -> CliMetadata {
        cot::cli::metadata!()
    }

    fn register_apps(&self, apps: &mut AppBuilder, _context: &RegisterAppsContext) {
        apps.register(DatabaseUserApp::new()); // Needed for admin authentication
        apps.register_with_views(AdminApp::new(), "/admin"); // Register the admin app
        apps.register_with_views(SwaggerUi::new(), "/swagger");
        apps.register_with_views(DemoAppCupsManagerApp, "");
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &MiddlewareContext,
    ) -> BoxedHandler {
        handler
            .middleware(StaticFilesMiddleware::from_context(context))
            .middleware(AuthMiddleware::new())
            .middleware(SessionMiddleware::new())
            .middleware(LiveReloadMiddleware::from_context(context))
            .build()
    }
}

#[cot::main]
fn main() -> impl Project {
    DemoAppCupsManagerProject
}
