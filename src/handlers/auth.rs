use axum::{
    extract::{Form, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use crate::auth::{clear_session, store_user_session, USER_ID_KEY};
use crate::models::User;
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_page() -> impl IntoResponse {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login - FastClient</title>
    <link rel="stylesheet" href="/assets/css/app.css">
    <script src="/assets/js/htmx.min.js"></script>
    <script src="/assets/js/htmx-ext-preload.js"></script>
    <script>
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
            document.documentElement.classList.add('dark');
        }
    </script>
</head>
<body class="bg-gray-50 min-h-screen dark:bg-warm-950" hx-ext="preload">
    <div class="flex min-h-full flex-col justify-center py-12 px-4 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
            <h1 class="text-center text-3xl font-bold tracking-tight text-gray-900 dark:text-warm-100">FastClient</h1>
            <h2 class="mt-2 text-center text-lg text-gray-600 dark:text-warm-400">Sign in to your account</h2>
        </div>
        <div class="mt-6 sm:mx-auto sm:w-full sm:max-w-md">
            <div class="bg-white dark:bg-warm-800 py-8 px-4 shadow-lg sm:rounded-xl sm:px-10">
                <form action="/login" method="post" class="space-y-6">
                    <input type="hidden" name="_csrf" value="">
                    <div>
                        <label for="email" class="form-label">Email address</label>
                        <input type="email" name="email" id="email" class="form-input" required autofocus>
                    </div>
                    <div>
                        <label for="password" class="form-label">Password</label>
                        <input type="password" name="password" id="password" class="form-input" required>
                    </div>
                    <button type="submit" class="btn btn-primary w-full">Sign in</button>
                </form>
                <p class="mt-6 text-center text-sm text-gray-600 dark:text-warm-400">
                    <a href="/register" class="font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300">Create one</a>
                </p>
            </div>
        </div>
    </div>
    <script src="/assets/js/app.js"></script>
</body>
</html>
    "#)
}

pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let user = match User::find_by_email(&state.pool, &form.email).await {
        Ok(Some(u)) => u,
        Ok(None) => return Html(r#"<div class="p-4 bg-red-100 text-red-800">Invalid email or password</div><a href="/login" class="btn btn-primary">Back</a>"#).into_response(),
        Err(_) => return Html(r#"<div class="p-4 bg-red-100 text-red-800">Database error</div><a href="/login">Back</a>"#).into_response(),
    };

    if !user.verify_password(&form.password) {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Invalid email or password</div><a href="/login">Back</a>"#).into_response();
    }

    if let Err(_) = store_user_session(&session, user.id).await {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Session error</div><a href="/login">Back</a>"#).into_response();
    }

    Redirect::to("/").into_response()
}

pub async fn logout(session: Session) -> impl IntoResponse {
    let _ = clear_session(&session).await;
    Redirect::to("/login")
}

pub async fn inject_user_id(
    session: Session,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_id: Option<Uuid> = session.get::<Uuid>(USER_ID_KEY).await.ok().flatten();
    if let Some(id) = user_id {
        req.extensions_mut().insert(id);
    }
    Ok(next.run(req).await)
}
