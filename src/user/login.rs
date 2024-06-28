use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::user::users::{SuperAdmin, SubAdmin, Staff, Technician, verify_password} ;
use crate::error::CustomError;
use crate::auth::claims::Claims;
use dotenvy::dotenv;
use std::env;
use log::error;


#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

pub async fn login(pool: web::Data<PgPool>, user: web::Json<LoginRequest>) -> impl Responder {
    let user_info = user.into_inner();
    
    let result = match find_user_by_email(&pool, &user_info.email).await {
        Ok(user) => match user {
            User::SuperAdmin(super_admin) => verify_user(super_admin.password, user_info.password, user_info.email),
            User::SubAdmin(sub_admin) => verify_user(sub_admin.password, user_info.password, user_info.email),
            User::Staff(staff) => verify_user(staff.password, user_info.password, user_info.email),
            User::Technician(technician) => verify_user(technician.password, user_info.password, user_info.email),
        },
        Err(e) => {
            error!("Invalid request: {:?}", e);
            HttpResponse::Unauthorized().body(format!("Invalid request: {:?}", e))
        }
    };

    result
}

async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<User, CustomError> {
    if let Ok(super_admin) = find_superadmin_by_email(pool, email).await {
        return Ok(User::SuperAdmin(super_admin));
    }
    if let Ok(sub_admin) = find_subadmin_by_email(pool, email).await {
        return Ok(User::SubAdmin(sub_admin));
    }
    if let Ok(staff) = find_staff_by_email(pool, email).await {
        return Ok(User::Staff(staff));
    }
    if let Ok(technician) = find_technician_by_email(pool, email).await {
        return Ok(User::Technician(technician));
    }

    Err(CustomError::OtherError("Not Found".to_string()))
}

fn verify_user(stored_password: String, input_password: String, email: String) -> HttpResponse {
    if verify_password(&stored_password, &input_password).is_ok() {
        let claims = Claims::with_email(&email);
        let secret = load_secret();
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap();
        HttpResponse::Ok().json(LoginResponse { token })
    } else {
        HttpResponse::Unauthorized().body("Invalid credentials")
    }
}

async fn find_superadmin_by_email(pool: &PgPool, email: &str) -> Result<SuperAdmin, CustomError> {
    let super_admin = sqlx::query_as!(
        SuperAdmin,
        "SELECT * FROM super_admin WHERE email = $1",
        email
    )
    .fetch_one(pool)
    .await?;
    Ok(super_admin)
}

async fn find_subadmin_by_email(pool: &PgPool, email: &str) -> Result<SubAdmin, CustomError> {
    let sub_admin = sqlx::query_as!(
        SubAdmin,
        "SELECT * FROM sub_admin WHERE email = $1",
        email
    )
    .fetch_one(pool)
    .await?;
    Ok(sub_admin)
}

async fn find_staff_by_email(pool: &PgPool, email: &str) -> Result<Staff, CustomError> {
    let staff = sqlx::query_as!(
        Staff,
        "SELECT * FROM staff WHERE email = $1",
        email
    )
    .fetch_one(pool)
    .await?;
    Ok(staff)
}

async fn find_technician_by_email(pool: &PgPool, email: &str) -> Result<Technician, CustomError> {
    let technician = sqlx::query_as!(
        Technician,
        "SELECT * FROM technician WHERE email = $1",
        email
    )
    .fetch_one(pool)
    .await?;
    Ok(technician)
}


fn load_secret() -> String {
    dotenv().ok();
    env::var("JWT_SECRET_KEY")
        .expect("JWT_SECRET_KEY must be set in the environment")
}

enum User {
    SuperAdmin(SuperAdmin),
    SubAdmin(SubAdmin),
    Staff(Staff),
    Technician(Technician),
}