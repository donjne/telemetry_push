use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::Utc;
use actix_web::{web, HttpResponse, Responder};
use sysinfo::System;
use rand::Rng;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MaintenanceRequest {
    pub maintenance_id: Option<i32>,
    pub reported_by_sub_admin_id: Option<i32>,
    pub reported_by_staff_id: Option<i32>,
    pub device_name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>, // Changed to Option<String>
    pub priority: Option<String>, // Changed to Option<String>
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

fn generate_id() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=i32::MAX)
}

pub async fn create_maintenance_request(
    pool: web::Data<PgPool>,
    request: web::Json<MaintenanceRequest>,
) -> impl Responder {
    let new_request = request.into_inner();

    let device_name = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let maintenance_id = new_request.maintenance_id.unwrap_or_else(generate_id);

    let maintenance_request = MaintenanceRequest {
        maintenance_id: Some(maintenance_id),
        reported_by_sub_admin_id: new_request.reported_by_sub_admin_id,
        reported_by_staff_id: new_request.reported_by_staff_id,
        device_name: Some(device_name),
        title: new_request.title,
        description: new_request.description,
        status: Some("Pending".to_string()), // Default status as string
        priority: Some("Medium".to_string()), // Default priority as string
        created_at: Some(Utc::now()),
        updated_at: None,
    };

    match save_maintenance_request_to_database(&pool, &maintenance_request).await {
        Ok(_) => HttpResponse::Created().json(maintenance_request),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create maintenance request"),
    }
}

async fn save_maintenance_request_to_database(
    pool: &PgPool,
    request: &MaintenanceRequest,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO maintenance_requests (maintenance_id, reported_by_sub_admin_id, reported_by_staff_id, device_name, title, description, status, priority, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        request.maintenance_id,
        request.reported_by_sub_admin_id,
        request.reported_by_staff_id,
        request.device_name,
        request.title,
        request.description,
        request.status, 
        request.priority,
        request.created_at,
        request.updated_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_maintenance_requests(
    pool: web::Data<PgPool>,
    reported_by_id: web::Path<i32>,
) -> impl Responder {
    let reported_by_id = reported_by_id.into_inner();
    match fetch_maintenance_requests_by_user(&pool, reported_by_id).await {
        Ok(requests) => HttpResponse::Ok().json(requests),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch maintenance requests"),
    }
}

pub async fn fetch_maintenance_requests_by_user(
    pool: &PgPool,
    reported_by_id: i32,
) -> Result<Vec<MaintenanceRequest>, sqlx::Error> {
    let requests = sqlx::query_as!(
        MaintenanceRequest,
        "SELECT * FROM maintenance_requests WHERE reported_by_sub_admin_id = $1 OR reported_by_staff_id = $1",
        reported_by_id
    )
    .fetch_all(pool)
    .await?;

    Ok(requests)
}

pub async fn get_user_specific_maintenance_request(
    pool: web::Data<PgPool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (reported_by_id, maintenance_id) = path.into_inner();
    match fetch_specific_maintenance_request_by_user(&pool, reported_by_id, maintenance_id).await {
        Ok(request) => HttpResponse::Ok().json(request),
        Err(_) => HttpResponse::NotFound().body("Maintenance request not found"),
    }
}

async fn fetch_specific_maintenance_request_by_user(
    pool: &PgPool,
    reported_by_id: i32,
    maintenance_id: i32,
) -> Result<MaintenanceRequest, sqlx::Error> {
    let request = sqlx::query_as!(
        MaintenanceRequest,
        "SELECT * FROM maintenance_requests WHERE (reported_by_sub_admin_id = $1 OR reported_by_staff_id = $1) AND maintenance_id = $2",
        reported_by_id,
        maintenance_id
    )
    .fetch_one(pool)
    .await?;

    Ok(request)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMaintenanceRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

pub async fn update_maintenance_request(
    pool: web::Data<PgPool>,
    path: web::Path<(i32, i32)>,
    update_request: web::Json<UpdateMaintenanceRequest>,
) -> impl Responder {
    let (reported_by_id, maintenance_id) = path.into_inner();
    let update_request = update_request.into_inner();
    let updated_at = Some(Utc::now());

    match update_maintenance_request_in_database(&pool, reported_by_id, maintenance_id, &update_request, updated_at).await {
        Ok(_) => HttpResponse::Ok().body("Maintenance request updated"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update maintenance request"),
    }
}

async fn update_maintenance_request_in_database(
    pool: &PgPool,
    reported_by_id: i32,
    maintenance_id: i32,
    update_request: &UpdateMaintenanceRequest,
    updated_at: Option<chrono::DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE maintenance_requests SET title = COALESCE($1, title), description = COALESCE($2, description), status = COALESCE($3, status), priority = COALESCE($4, priority), updated_at = $5 WHERE (reported_by_sub_admin_id = $6 OR reported_by_staff_id = $6) AND maintenance_id = $7",
        update_request.title,
        update_request.description,
        update_request.status,
        update_request.priority,
        updated_at,
        reported_by_id,
        maintenance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_maintenance_request(
    pool: web::Data<PgPool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (reported_by_id, maintenance_id) = path.into_inner();

    match delete_maintenance_request_from_database(&pool, reported_by_id, maintenance_id).await {
        Ok(_) => HttpResponse::Ok().body("Maintenance request deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete maintenance request"),
    }
}

async fn delete_maintenance_request_from_database(
    pool: &PgPool,
    reported_by_id: i32,
    maintenance_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM maintenance_requests WHERE (reported_by_sub_admin_id = $1 OR reported_by_staff_id = $1) AND maintenance_id = $2",
        reported_by_id,
        maintenance_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn count_ongoing_maintenance_requests(
    pool: &PgPool,
    reported_by_id: i32,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM maintenance_requests WHERE (reported_by_sub_admin_id = $1 OR reported_by_staff_id = $1) AND status = 'Ongoing'",
        reported_by_id
    )
    .fetch_one(pool)
    .await?
    .count;

    Ok(count.unwrap())
}

pub async fn get_ongoing_maintenance_count(
    pool: web::Data<PgPool>,
    reported_by_id: web::Path<i32>,
) -> impl Responder {
    let reported_by_id = reported_by_id.into_inner();

    match count_ongoing_maintenance_requests(&pool, reported_by_id).await {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(_) => HttpResponse::InternalServerError().body("Failed to count ongoing maintenance requests"),
    }
}
