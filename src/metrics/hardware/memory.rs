use sysinfo::System;
use serde::Serialize;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use actix_web::{web, HttpResponse, Responder};

#[derive(Serialize, Clone)]
pub struct MemoryMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub total_memory: Option<f64>,
    pub used_memory: Option<f64>,
}

impl MemoryMetrics {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, total_memory: Option<f64>, used_memory: Option<f64>) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            total_memory,
            used_memory,
        }
    }

    pub fn update(&mut self, new_total_memory: Option<f64>, new_used_memory: Option<f64>) {
        self.total_memory = new_total_memory;
        self.used_memory = new_used_memory;
    }
}

pub async fn get_memory_info_handler(pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();
    match get_memory_info_for_user(&pool, user_id_value).await {
    Ok(mut memory_metrics) => {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = Some(sys.total_memory() as f64).map(|total| total / 1_048_576.0);
    let used_memory = Some(sys.used_memory() as f64).map(|used| used / 1_048_576.0);
    memory_metrics.update(total_memory, used_memory);
    if let Err(e) = save_memory_metrics_to_database(&pool, &memory_metrics).await {
        eprintln!("Failed to save metrics to database: {}", e);
        return HttpResponse::InternalServerError().body("Failed to save Memory info");
    }   
    HttpResponse::Ok().json(memory_metrics)
    }
    Err(_) => {
        let new_memory_metrics = gather_memory_metrics(Some(user_id_value), None).await;
        if let Err(e) = save_memory_metrics_to_database(&pool, &new_memory_metrics).await {
            eprintln!("Failed to save metrics to database: {}", e);
            return HttpResponse::InternalServerError().body("Failed to save CPU info");
        }
        HttpResponse::Ok().json(new_memory_metrics)
    }
}
}

pub async fn gather_memory_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> MemoryMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let mut sys = System::new_all();
    sys.refresh_all();
    let total_memory = Some(sys.total_memory() as f64).map(|total| total / 1_048_576.0);
    let used_memory = Some(sys.used_memory() as f64).map(|used| used / 1_048_576.0);
    return MemoryMetrics::new(sub_admin_metrics_id, staff_metrics_id, total_memory, used_memory)
}
}

pub async fn save_memory_metrics_to_database(pool: &PgPool, metrics: &MemoryMetrics) -> Result<(), Box<dyn std::error::Error>> {
    
        sqlx::query!(
            "INSERT INTO memory_metrics (sub_admin_metrics_id, staff_metrics_id, total_memory, used_memory) VALUES ($1, $2, $3, $4)",
            metrics.sub_admin_metrics_id,
            metrics.staff_metrics_id,
            metrics.total_memory,
            metrics.used_memory,
        )
        .execute(pool)
        .await?;
    Ok(())

    }


pub async fn get_memory_info(pool: &PgPool) -> Result<Vec<MemoryMetrics>, sqlx::Error> {
    let rows = sqlx::query_as!(
        MemoryMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, total_memory, used_memory FROM memory_metrics"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn get_memory_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<MemoryMetrics, sqlx::Error> {
    sqlx::query_as!(
        MemoryMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, total_memory, used_memory 
        FROM memory_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}