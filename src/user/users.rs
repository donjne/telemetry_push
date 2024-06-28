use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Sha512, Digest};
use std::error::Error;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::CustomError;
use chrono::Utc;
use log::error;
use rand::Rng;
use crate::metrics::hardware::{
aboutsys::{gather_system_info, save_systeminfo_metrics_to_database}, 
cpu::{gather_cpu_metrics, save_cpu_metrics_to_database},
disk::{gather_disk_metrics, save_disk_metrics_to_database},
memory::{gather_memory_metrics, save_memory_metrics_to_database},
network::{gather_network_metrics, save_network_metrics_to_database},
};

use crate::metrics::software::{
filesystem::{gather_filesystem_metrics, save_filesystem_metrics_to_database},
ip_location::{gather_ip_location, save_ip_location_to_database},
process::{gather_process_metrics, save_process_metrics_to_database},
services::{gather_services_status_metrics, save_service_status_to_database},
uptime::{gather_uptime_metrics, save_uptime_metrics_to_database},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SuperAdmin {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Staff {
    pub id: Option<i32>,
    pub metrics_id: Option<i32>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub company_affiliated_to: Option<String>, // New field
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Technician {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubAdmin {
    pub id: Option<i32>,
    pub metrics_id: Option<i32>,
    pub company_name: Option<String>,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
    SuperAdmin,
    #[serde(rename = "Subadmin")]
    SubAdmin,
    Staff,
    Technician,
}

fn generate_id() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=i32::MAX)
}

pub async fn createsuper(pool: web::Data<PgPool>, user: web::Json<SuperAdmin>) -> impl Responder {
    let new_user = user.into_inner();

    if !is_email_valid(&new_user.email) {
        return HttpResponse::BadRequest().body("Invalid email format");
    }

    let hashed_password = match multi_scheme_hash(&new_user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let super_admin = SuperAdmin {
        id: new_user.id,
        name: new_user.name,
        email: new_user.email,
        password: hashed_password,
        created_at: Some(Utc::now()),
        updated_at: None,
    };

    match save_superadmin_to_database(&pool, &super_admin).await {
        Ok(_) => HttpResponse::Created().body("Super admin created successfully"),
        Err(e) => {
            error!("Failed to create super admin: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to create super admin: {:?}", e))
        },
    }
}

pub async fn createsub(pool: web::Data<PgPool>, user: web::Json<SubAdmin>) -> impl Responder {
    let new_user = user.into_inner();

    if !is_email_valid(&new_user.email) {
        return HttpResponse::BadRequest().body("Invalid email format");
    }

    let hashed_password = match multi_scheme_hash(&new_user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let sub_admin = SubAdmin {
        id: Some(new_user.id.unwrap_or_else(generate_id)),
        metrics_id: Some(new_user.metrics_id.unwrap_or_else(generate_id)),
        company_name: new_user.company_name,
        email: new_user.email,
        phone: new_user.phone,
        password: hashed_password,
        created_at: Some(Utc::now()),
        updated_at: None,
    };

    match save_subadmin_to_database(&pool, &sub_admin).await {
        Ok(_) => {
            // Gather and save system info
            let system_info = gather_system_info(sub_admin.metrics_id, None).await;
            if let Err(e) = save_systeminfo_metrics_to_database(&pool, &system_info).await {
                error!("Failed to save system info: {:?}", e);
                return HttpResponse::InternalServerError().body(format!("Failed to save system info: {:?}", e));
            }

            // Gather and save CPU info
            let cpu_info = gather_cpu_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_cpu_metrics_to_database(&pool, &cpu_info).await {
            error!("Failed to save CPU info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save CPU info: {:?}", e));
            }
            
        // Gather and save Disk info
        let disk_info = gather_disk_metrics(sub_admin.metrics_id, None).await;
        if let Err(e) = save_disk_metrics_to_database(&pool, &disk_info).await {
        error!("Failed to save CPU info: {:?}", e);
        return HttpResponse::InternalServerError().body(format!("Failed to save Disk info: {:?}", e));
        }

            // Gather and save Disk info
            let memory_info = gather_memory_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_memory_metrics_to_database(&pool, &memory_info).await {
            error!("Failed to save CPU info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Memory info: {:?}", e));
            }

            // Gather and save Network info
            let disk_info = gather_network_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_network_metrics_to_database(&pool, &disk_info).await {
            error!("Failed to save disk info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Memory info: {:?}", e));
            }

            // Gather and save Filesystem info
            let filesystem_info = gather_filesystem_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_filesystem_metrics_to_database(&pool, &filesystem_info).await {
            error!("Failed to save filesystem info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Memory info: {:?}", e));
            }

            let ip_info = gather_ip_location(sub_admin.metrics_id, None).await;
            if let Err(e) = save_ip_location_to_database(&pool, &ip_info).await {
            error!("Failed to save ip info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save ip info: {:?}", e));
            }

            let process_info = gather_process_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_process_metrics_to_database(&pool, &process_info).await {
            error!("Failed to save process info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save process info: {:?}", e));
            }

            let services_info = gather_services_status_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_service_status_to_database(&pool, &services_info).await {
            error!("Failed to save services info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save services info: {:?}", e));
            }

            let uptime_info = gather_uptime_metrics(sub_admin.metrics_id, None).await;
            if let Err(e) = save_uptime_metrics_to_database(&pool, &uptime_info).await {
            error!("Failed to save process info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save process info: {:?}", e));
            }

        HttpResponse::Created().body("Sub admin created successfully")
        },

        Err(e) => {
            error!("Failed to create sub admin: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to create sub admin: {:?}", e))
        }
    }
}

pub async fn createstaff(pool: web::Data<PgPool>, user: web::Json<Staff>) -> impl Responder {
    let new_user = user.into_inner();

    if !is_email_valid(&new_user.email) {
        return HttpResponse::BadRequest().body("Invalid email format");
    }

    let hashed_password = match multi_scheme_hash(&new_user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let staff = Staff {
        id: Some(new_user.id.unwrap_or_else(generate_id)),
        metrics_id: Some(new_user.metrics_id.unwrap_or_else(generate_id)),
        name: new_user.name,
        email: new_user.email,
        password: hashed_password,
        created_at: Some(Utc::now()),
        updated_at: None,
        company_affiliated_to: new_user.company_affiliated_to
    };

    match save_staff_to_database(&pool, &staff).await {
        Ok(_) => {
            // Gather and save system info
            let system_info = gather_system_info(None, staff.metrics_id).await;
            if let Err(e) = save_systeminfo_metrics_to_database(&pool, &system_info).await {
                error!("Failed to save system info: {:?}", e);
                return HttpResponse::InternalServerError().body(format!("Failed to save system info: {:?}", e));
            }
    
            // Gather and save CPU info
            let cpu_info = gather_cpu_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_cpu_metrics_to_database(&pool, &cpu_info).await {
                error!("Failed to save CPU info: {:?}", e);
                return HttpResponse::InternalServerError().body(format!("Failed to save CPU info: {:?}", e));
            }

            // Gather and save Disk info
            let disk_info = gather_disk_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_disk_metrics_to_database(&pool, &disk_info).await {
            error!("Failed to save CPU info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Disk info: {:?}", e));
            }

            // Gather and save Memory info
            let memory_info = gather_memory_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_memory_metrics_to_database(&pool, &memory_info).await {
            error!("Failed to save CPU info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Memory info: {:?}", e));
            }

            // Gather and save Network info
            let network_info = gather_network_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_network_metrics_to_database(&pool, &network_info).await {
            error!("Failed to save CPU info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Memory info: {:?}", e));
            }

            // Gather and save Network info
            let filesystem_info = gather_filesystem_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_filesystem_metrics_to_database(&pool, &filesystem_info).await {
            error!("Failed to save CPU info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save Memory info: {:?}", e));
            }

            let ip_info = gather_ip_location(None, staff.metrics_id).await;
            if let Err(e) = save_ip_location_to_database(&pool, &ip_info).await {
            error!("Failed to save ip info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save ip info: {:?}", e));
            }

            let process_info = gather_process_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_process_metrics_to_database(&pool, &process_info).await {
            error!("Failed to save process info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save process info: {:?}", e));
            }

            let services_info = gather_services_status_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_service_status_to_database(&pool, &services_info).await {
            error!("Failed to save services info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save services info: {:?}", e));
            }

            let uptime_info = gather_uptime_metrics(None, staff.metrics_id).await;
            if let Err(e) = save_uptime_metrics_to_database(&pool, &uptime_info).await {
            error!("Failed to save process info: {:?}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to save process info: {:?}", e));
            }
    
            HttpResponse::Created().body("Staff created successfully")
        },

        Err(e) => {
            error!("Failed to create staff: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to create staff: {:?}", e))
        },
    }
    
}

pub async fn createtechnician(pool: web::Data<PgPool>, user: web::Json<Technician>) -> impl Responder {
    let new_user = user.into_inner();

    if !is_email_valid(&new_user.email) {
        return HttpResponse::BadRequest().body("Invalid email format");
    }

    let hashed_password = match multi_scheme_hash(&new_user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let technician = Technician {
        id: new_user.id,
        name: new_user.name,
        email: new_user.email,
        password: hashed_password,
        created_at: Some(Utc::now()),
        updated_at: None,
    };

    match save_technician_to_database(&pool, &technician).await {
        Ok(_) => HttpResponse::Created().body("Technician created successfully"),
        Err(e) => {
            error!("Failed to create technician admin: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to create technician: {:?}", e))
        },
    }
}


fn is_email_valid(email: &str) -> bool {
    email.contains('@')
}
fn multi_scheme_hash(password: &str) -> Result<String, CustomError> {
    // First, hash the password using SHA-512
    let mut hasher = Sha512::new();
    hasher.update(password.as_bytes());
    let sha512_hash = hasher.finalize();
    let sha512_hash_bytes = sha512_hash.as_slice();

    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    // Hash the SHA-512 hash using Argon2
    argon2.hash_password(sha512_hash_bytes, &salt)
    .map(|password_hash| password_hash.to_string())
    .map_err(|e| CustomError::from(e))
}

pub fn verify_password(hash: &str, password: &str) -> Result<(), Box<dyn Error>> {
    // First, hash the password using SHA-512
    let mut hasher = Sha512::new();
    hasher.update(password.as_bytes());
    let sha512_hash = hasher.finalize();

    // Convert SHA-512 hash to a byte slice
    let sha512_hash_bytes = sha512_hash.as_slice();

    // Parse the Argon2 hash
    let parsed_hash = PasswordHash::new(hash).unwrap();

    // Verify the SHA-512 hash using Argon2
    Argon2::default().verify_password(sha512_hash_bytes, &parsed_hash).unwrap();

    Ok(())
}

async fn save_subadmin_to_database(pool: &PgPool, user: &SubAdmin) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "INSERT INTO sub_admin (id, metrics_id, company_name, email, phone, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        user.id, user.metrics_id, user.company_name, user.email, user.phone, user.password, user.created_at, user.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn save_staff_to_database(pool: &PgPool, user: &Staff) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "INSERT INTO staff (id, metrics_id, name, email, password, created_at, updated_at, company_affiliated_to) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        user.id, user.metrics_id, user.name, user.email, user.password, user.created_at, user.updated_at, user.company_affiliated_to
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn save_technician_to_database(pool: &PgPool, user: &Technician) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "INSERT INTO technician (id, name, email, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        user.id, user.name, user.email, user.password, user.created_at, user.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn save_superadmin_to_database(pool: &PgPool, user: &SuperAdmin) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "INSERT INTO super_admin (id, name, email, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        user.id, user.name, user.email, user.password, user.created_at, user.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_staffs_by_company(
    pool: web::Data<PgPool>,
    sub_admin: web::Json<SubAdmin>,
) -> impl Responder {
    let company_name = match &sub_admin.company_name {
        Some(name) => name,
        None => return HttpResponse::BadRequest().body("Sub-admin must have a company name"),
    };

    match fetch_all_staffs_by_company(&pool, company_name).await {
        Ok(staffs) => HttpResponse::Ok().json(staffs),
        Err(e) => {
            error!("Failed to fetch staffs: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch staffs")
        }
    }
}

async fn fetch_all_staffs_by_company(pool: &PgPool, company_name: &str) -> Result<Vec<Staff>, Box<dyn std::error::Error>> {
    let staffs = sqlx::query_as!(
        Staff,
        "SELECT * FROM staff WHERE company_affiliated_to = $1",
        company_name
    )
    .fetch_all(pool)
    .await?;
    
    Ok(staffs)
}

pub async fn count_staffs_by_company(
    pool: web::Data<PgPool>,
    sub_admin: web::Json<SubAdmin>,
) -> impl Responder {
    let company_name = match &sub_admin.company_name {
        Some(name) => name,
        None => return HttpResponse::BadRequest().body("Sub-admin must have a company name"),
    };

    match count_staffs_in_company(&pool, company_name).await {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(e) => {
            error!("Failed to count staffs: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to count staffs")
        }
    }
}

async fn count_staffs_in_company(pool: &PgPool, company_name: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM staff WHERE company_affiliated_to = $1",
        company_name
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);

    Ok(count)
}

pub async fn get_all_sub_admins(
    pool: web::Data<PgPool>,
) -> impl Responder {
    match fetch_all_sub_admins(&pool).await {
        Ok(sub_admins) => HttpResponse::Ok().json(sub_admins),
        Err(e) => {
            error!("Failed to fetch sub-admins: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch sub-admins")
        }
    }
}

async fn fetch_all_sub_admins(pool: &PgPool) -> Result<Vec<SubAdmin>, Box<dyn std::error::Error>> {
    let sub_admins = sqlx::query_as!(
        SubAdmin,
        "SELECT * FROM sub_admin"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(sub_admins)
}

pub async fn count_sub_admins(
    pool: web::Data<PgPool>,
) -> impl Responder {
    match count_all_sub_admins(&pool).await {
        Ok(count) => HttpResponse::Ok().json(count),
        Err(e) => {
            error!("Failed to count sub-admins: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to count sub-admins")
        }
    }
}

async fn count_all_sub_admins(pool: &PgPool) -> Result<i64, Box<dyn std::error::Error>> {
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM sub_admin"
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);

    Ok(count)
}