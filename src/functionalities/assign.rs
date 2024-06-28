// use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::Utc;
use actix_web::{web, HttpResponse, Responder};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SystemAssignment {
    pub staff_full_name: Option<String>,
    pub staff_department: Option<String>,
    pub staff_role_and_position: Option<String>,
    pub system_name: Option<String>,
    pub new_system_id: Option<String>,
    pub operating_system: Option<String>,
    pub return_date: Option<String>,
    pub assigned_by: Option<String>,
    pub purpose: Option<String>,
    pub sub_admin_id_email: Option<String>,
    pub staff_id_email: Option<String>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

pub async fn create_system_assignment(
    pool: web::Data<PgPool>,
    assignment: web::Json<SystemAssignment>,
) -> impl Responder {
    let new_assignment = assignment.into_inner();

    let system_assignment = SystemAssignment {
        created_at: Some(Utc::now()),
        updated_at: None,
        ..new_assignment
    };

    match save_system_assignment_to_database(&pool, &system_assignment).await {
        Ok(_) => HttpResponse::Created().json(system_assignment),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create system assignment"),
    }
}

async fn save_system_assignment_to_database(
    pool: &PgPool,
    assignment: &SystemAssignment,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO system_assignments (staff_full_name, staff_department, staff_role_and_position, system_name, new_system_id, operating_system, return_date, assigned_by, purpose, sub_admin_id_email, staff_id_email, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        assignment.staff_full_name,
        assignment.staff_department,
        assignment.staff_role_and_position,
        assignment.system_name,
        assignment.new_system_id,
        assignment.operating_system,
        assignment.return_date,
        assignment.assigned_by,
        assignment.purpose,
        assignment.sub_admin_id_email,
        assignment.staff_id_email,
        assignment.created_at,
        assignment.updated_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_system_assignment_by_id(
    pool: &PgPool,
    new_system_id: &str,
) -> Result<SystemAssignment, sqlx::Error> {
    let assignment = sqlx::query_as!(
        SystemAssignment,
        "SELECT * FROM system_assignments WHERE new_system_id = $1",
        new_system_id
    )
    .fetch_one(pool)
    .await?;

    Ok(assignment)
}


pub async fn get_system_assignment(
    pool: web::Data<PgPool>,
    new_system_id: web::Path<String>,
) -> impl Responder {
    let new_system_id = new_system_id.into_inner();
    match fetch_system_assignment_by_id(&pool, &new_system_id).await {
        Ok(assignment) => HttpResponse::Ok().json(assignment),
        Err(_) => HttpResponse::NotFound().body("System assignment not found"),
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSystemAssignment {
    pub staff_full_name: Option<String>,
    pub staff_department: Option<String>,
    pub staff_role_and_position: Option<String>,
    pub system_name: Option<String>,
    pub operating_system: Option<String>,
    pub return_date: Option<String>,
    pub assigned_by: Option<String>,
    pub purpose: Option<String>,
    pub sub_admin_id_email: Option<String>,
    pub staff_id_email: Option<String>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

pub async fn update_system_assignment(
    pool: web::Data<PgPool>,
    new_system_id: web::Path<String>,
    update_assignment: web::Json<UpdateSystemAssignment>,
) -> impl Responder {
    let new_system_id = new_system_id.into_inner();
    let update_assignment = update_assignment.into_inner();
    let updated_at = Some(Utc::now());

    match update_system_assignment_in_database(&pool, &new_system_id, &update_assignment, updated_at).await {
        Ok(_) => HttpResponse::Ok().body("System assignment updated"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update system assignment"),
    }
}

async fn update_system_assignment_in_database(
    pool: &PgPool,
    new_system_id: &str,
    update_assignment: &UpdateSystemAssignment,
    updated_at: Option<chrono::DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE system_assignments SET staff_full_name = COALESCE($1, staff_full_name), staff_department = COALESCE($2, staff_department), staff_role_and_position = COALESCE($3, staff_role_and_position), system_name = COALESCE($4, system_name), operating_system = COALESCE($5, operating_system), return_date = COALESCE($6, return_date), assigned_by = COALESCE($7, assigned_by), purpose = COALESCE($8, purpose), sub_admin_id_email = COALESCE($9, sub_admin_id_email), staff_id_email = COALESCE($10, staff_id_email), updated_at = $11 WHERE new_system_id = $12",
        update_assignment.staff_full_name,
        update_assignment.staff_department,
        update_assignment.staff_role_and_position,
        update_assignment.system_name,
        update_assignment.operating_system,
        update_assignment.return_date,
        update_assignment.assigned_by,
        update_assignment.purpose,
        update_assignment.sub_admin_id_email,
        update_assignment.staff_id_email,
        updated_at,
        new_system_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_system_assignment(
    pool: web::Data<PgPool>,
    new_system_id: web::Path<String>,
) -> impl Responder {
    let new_system_id = new_system_id.into_inner();

    match delete_system_assignment_from_database(&pool, &new_system_id).await {
        Ok(_) => HttpResponse::Ok().body("System assignment deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete system assignment"),
    }
}

async fn delete_system_assignment_from_database(
    pool: &PgPool,
    new_system_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM system_assignments WHERE new_system_id = $1",
        new_system_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_system_assignment_count(
    pool: web::Data<PgPool>,
    email: web::Path<String>,
) -> impl Responder {
    let email = email.into_inner();
    match fetch_system_assignment_count_by_email(&pool, &email).await {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch system assignment count"),
    }
}

async fn fetch_system_assignment_count_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM system_assignments WHERE sub_admin_id_email = $1 OR staff_id_email = $1",
        email
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);

    Ok(count)
}
