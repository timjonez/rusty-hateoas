use crate::app::AppState;
use crate::contacts::forms::ContactForm;
use crate::contacts::models::Contact;
use axum::extract::{Path, Query, RawForm, State};
use axum::http::header::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use std::vec::Vec;
use tera::Context;

use std::collections::HashMap;
use std::sync::Arc;

pub async fn app_contacts(State(app): State<Arc<AppState>>) -> String {
    let contacts = Contact::all(&app.db, 0).await.unwrap();
    let mut context = Context::new();
    context.insert("contacts", &contacts);
    let res = app.tera.render("hv/app.xml", &context).unwrap();
    res.to_string()
}

pub async fn contacts(
    headers: HeaderMap,
    State(app): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let page: i32 = match params.contains_key("page") {
        true => params.get("page").unwrap().parse().unwrap(),
        false => 1,
    };
    let offset: i64 = ((page - 1) * 5).into();
    let contacts = match params.get("q") {
        Some(q) => Contact::search(&app.db, q.to_string()).await.unwrap(),
        None => Contact::all(&app.db, offset).await.unwrap(),
    };
    let mut context = Context::new();
    context.insert("page", &page);
    context.insert("contacts", &contacts);
    if headers.contains_key("hx-trigger") {
        if headers.get("hx-trigger").unwrap() == "search" {
            return Html(app.tera.render("contacts/_rows.html", &context).unwrap());
        }
    }
    Html(app.tera.render("contacts/list.html", &context).unwrap())
}

pub async fn num_contacts(State(app): State<Arc<AppState>>) -> String {
    let count = Contact::count(&app.db).await;
    format!("{} total contacts", count)
}

pub async fn get_contact(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Html<String> {
    let mut context = Context::new();
    let contact = Contact::get(&app.db, user_id).await.unwrap();
    context.insert("contact", &contact);
    Html(app.tera.render("contacts/detail.html", &context).unwrap())
}

pub async fn validate_email(
    State(app): State<Arc<AppState>>,
    Query(args): Query<HashMap<String, String>>,
) -> String {
    let email = args.get("email").unwrap();
    let contacts = Contact::filter_email(&app.db, email).await.unwrap();
    match contacts.len() {
        0 => return String::new(),
        _ => return String::from("This email is already taken"),
    }
}

pub async fn get_edit_contact(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Html<String> {
    let mut context = Context::new();
    let contact = Contact::get(&app.db, user_id).await.unwrap();
    let form = ContactForm::from(&contact);
    context.insert("form", &form);
    context.insert("contact", &contact);
    context.insert("errors", &HashMap::<String, String>::new());
    Html(app.tera.render("contacts/edit.html", &context).unwrap())
}

pub async fn edit_contact(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Form(form): Form<ContactForm>,
) -> Response {
    let mut context = Context::new();
    let (valid, errors) = form.is_valid();
    if valid {
        let _ = form.update(user_id, &app.db).await;
        return Redirect::to("/contacts").into_response();
    }
    context.insert("errors", &errors);
    context.insert("form", &form);
    Html(app.tera.render("contacts/create.html", &context).unwrap()).into_response()
}

pub async fn get_create_contact(State(app): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("form", &ContactForm::new());
    context.insert("errors", &HashMap::<String, String>::new());
    Html(app.tera.render("contacts/create.html", &context).unwrap())
}

pub async fn create_contact(
    State(app): State<Arc<AppState>>,
    Form(form): Form<ContactForm>,
) -> Response {
    let mut context = Context::new();
    let (valid, errors) = form.is_valid();
    if valid {
        let _ = form.save(&app.db).await;
        return Redirect::to("/contacts").into_response();
    }
    context.insert("errors", &errors);
    context.insert("form", &form);
    Html(app.tera.render("contacts/create.html", &context).unwrap()).into_response()
}

pub async fn delete_contact(
    headers: HeaderMap,
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Html<String> {
    let mut context = Context::new();
    let contact = Contact::get(&app.db, user_id).await.unwrap();
    context.insert("contact", &contact);
    Html(app.tera.render("contacts/detail.html", &context).unwrap())
}


pub async fn delete_contact_list(State(app): State<Arc<AppState>>, raw_form: RawForm) -> Response {
    let mut ids: Vec<i32> = Vec::new();
    let str_form = match std::str::from_utf8(&raw_form.0) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    for i in str_form.split("&") {
        let parts: Vec<&str> = i.split("=").collect();
        if parts.len() == 2 && parts.first().unwrap() == &"selected_contact_ids" {
            let str_id = parts.last().unwrap();
            match str_id.parse::<i32>() {
                Ok(id) => ids.push(id.into()),
                Err(_) => {}
            }
        }
    }

    for id in ids {
        match Contact::get(&app.db, id).await {
            Err(_) => {}
            Ok(contact) => {
                let _ = contact.delete(&app.db).await;
            }
        }
    }
    return Redirect::to("/contacts").into_response();
}
