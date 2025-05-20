mod cup;
mod migrations;
mod qr;

use crate::cup::*;
use async_trait::async_trait;
use cot::admin::{AdminApp, AdminModelManager, DefaultAdminModelManager};
use cot::auth::db::{DatabaseUser, DatabaseUserApp};
use cot::cli::CliMetadata;
use cot::common_types::Password;
use cot::db::migrations::SyncDynMigration;
use cot::middleware::{AuthMiddleware, LiveReloadMiddleware, SessionMiddleware};
use cot::openapi::swagger_ui::SwaggerUi;
use cot::project::{MiddlewareContext, RegisterAppsContext, RootHandlerBuilder};
use cot::router::method::openapi::api_post;
use cot::router::method::{get, post};
use cot::router::{Route, Router};
use cot::static_files::{StaticFile, StaticFilesMiddleware};
use cot::{static_files, App, AppBuilder, BoxedHandler, Project, ProjectContext};

struct DemoAppCupsManagerApp;

#[async_trait]
impl App for DemoAppCupsManagerApp {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }
    async fn init(&self, context: &mut ProjectContext) -> cot::Result<()> {
        // Check if admin user exists
        let admin_username = std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
        let user = DatabaseUser::get_by_username(context.database(), &admin_username).await?;
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
            Route::with_handler_and_name("/", get(create_cup_page), "index"),
            Route::with_handler_and_name("/cup/form", post(create_cup_form), "create-cup-form"),
            Route::with_handler_and_name("/cup/{id}", get(get_cup), "get-cup"),
        ])
    }

    fn migrations(&self) -> Vec<Box<SyncDynMigration>> {
        cot::db::migrations::wrap_migrations(migrations::MIGRATIONS)
    }

    fn admin_model_managers(&self) -> Vec<Box<dyn AdminModelManager>> {
        vec![Box::new(DefaultAdminModelManager::<Cup>::new())]
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
