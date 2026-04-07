use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::auth::store_user_session;
use crate::models::user::CreateUser;
use crate::models::User;
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterForm {
    name: String,
    email: String,
    password: String,
    password_confirmation: String,
}

pub async fn register_page() -> impl IntoResponse {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Register - FastClient</title>
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
            <h2 class="mt-2 text-center text-lg text-gray-600 dark:text-warm-400">Create your account</h2>
        </div>
        <div class="mt-6 sm:mx-auto sm:w-full sm:max-w-md">
            <div class="bg-white dark:bg-warm-800 py-8 px-4 shadow-lg sm:rounded-xl sm:px-10">
                <form action="/register" method="post" class="space-y-6">
                    <input type="hidden" name="_csrf" value="">
                    <div>
                        <label for="name" class="form-label">Full Name</label>
                        <input type="text" name="name" id="name" class="form-input" required autofocus>
                    </div>
                    <div>
                        <label for="email" class="form-label">Email address</label>
                        <input type="email" name="email" id="email" class="form-input" required>
                    </div>
                    <div>
                        <label for="password" class="form-label">Password</label>
                        <input type="password" name="password" id="password" class="form-input" required>
                    </div>
                    <div>
                        <label for="password_confirmation" class="form-label">Confirm Password</label>
                        <input type="password" name="password_confirmation" id="password_confirmation" class="form-input" required>
                    </div>
                    <button type="submit" class="btn btn-primary w-full">Create Account</button>
                </form>
                <p class="mt-6 text-center text-sm text-gray-600 dark:text-warm-400">
                    <a href="/login" class="font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300">Sign in</a>
                </p>
            </div>
        </div>
    </div>
    <script src="/assets/js/app.js"></script>
</body>
</html>
    "#)
}

pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    // Validation
    if form.name.trim().is_empty() {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Name is required</div><a href="/register" class="btn btn-primary">Back</a>"#).into_response();
    }
    if form.email.trim().is_empty() {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Email is required</div><a href="/register">Back</a>"#).into_response();
    }
    if !form.email.contains('@') {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Please enter a valid email address</div><a href="/register">Back</a>"#).into_response();
    }
    if let Ok(Some(_)) = User::find_by_email(&state.pool, &form.email).await {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">This email is already registered</div><a href="/register">Back</a>"#).into_response();
    }
    if form.password.is_empty() {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Password is required</div><a href="/register">Back</a>"#).into_response();
    }
    if form.password.len() < 8 {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Password must be at least 8 characters</div><a href="/register">Back</a>"#).into_response();
    }
    if form.password != form.password_confirmation {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Passwords do not match</div><a href="/register">Back</a>"#).into_response();
    }

    let user = match User::create(&state.pool, &CreateUser {
        email: form.email.clone(),
        password: form.password,
        name: form.name.clone(),
    }).await {
        Ok(user) => user,
        Err(e) => return Html(format!(r#"<div class="p-4 bg-red-100 text-red-800">Registration error: {}</div><a href="/register">Back</a>"#, e)).into_response(),
    };

    if let Err(_) = store_user_session(&session, user.id).await {
        return Html(r#"<div class="p-4 bg-red-100 text-red-800">Session error</div><a href="/register">Back</a>"#).into_response();
    }

    Redirect::to("/").into_response()
}
