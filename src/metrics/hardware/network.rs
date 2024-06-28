use sysinfo::Networks;
use serde::Serialize;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use actix_web::{web, HttpResponse, Responder};


#[derive(Serialize, Clone)]
pub struct NetworkMetrics {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub total_received: Option<i32>,
    pub total_transmitted: Option<i32>,
}

impl NetworkMetrics {
    pub fn new(sub_admin_metrics_id: Option<i32>, staff_metrics_id: Option<i32>, total_received: Option<i32>, total_transmitted: Option<i32>) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            total_received,
            total_transmitted,
        }
    }

    pub fn update(&mut self, new_total_received: Option<i32>, new_total_transmitted: Option<i32>) {
        self.total_received = new_total_received;
        self.total_transmitted = new_total_transmitted;
    }
}

pub async fn get_network_info_handler(pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();
    match get_network_info_for_user(&pool, user_id_value).await {
        Ok(mut network_metrics) => {
        let networks = Networks::new_with_refreshed_list();
        let total_received = networks.values().map(|n| n.received() as i32).sum();
        let total_transmitted = networks.values().map(|n| n.transmitted() as i32).sum();

        let total_received = Some(total_received);
        let total_transmitted = Some(total_transmitted);
        network_metrics.update(total_received, total_transmitted);
        if let Err(e) = save_network_metrics_to_database(&pool, &network_metrics).await {
            eprintln!("Failed to save metrics to database: {}", e);
            return HttpResponse::InternalServerError().body("Failed to save Memory info");
        }   
        HttpResponse::Ok().json(network_metrics)
    }    
    Err(_) => {
        let new_network_metrics = gather_network_metrics(Some(user_id_value), None).await;
        if let Err(e) = save_network_metrics_to_database(&pool, &new_network_metrics).await {
            eprintln!("Failed to save metrics to database: {}", e);
            return HttpResponse::InternalServerError().body("Failed to save CPU info");
        }
        HttpResponse::Ok().json(new_network_metrics)
    }
}
}
pub async fn gather_network_metrics(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> NetworkMetrics {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour
    loop {
    interval.tick().await;
    let networks = Networks::new_with_refreshed_list();
    let total_received = networks.values().map(|n| n.received() as i32).sum();
    let total_transmitted = networks.values().map(|n| n.transmitted() as i32).sum();
    let total_received = Some(total_received);
    let total_transmitted = Some(total_transmitted);
    return NetworkMetrics::new(sub_admin_metrics_id, staff_metrics_id, total_received, total_transmitted)
}
}

pub async fn save_network_metrics_to_database(pool: &PgPool, metrics: &NetworkMetrics) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "INSERT INTO network_metrics (sub_admin_metrics_id, staff_metrics_id, total_received, total_transmitted) VALUES ($1, $2, $3, $4)",
            metrics.sub_admin_metrics_id,
            metrics.staff_metrics_id,
            metrics.total_received,
            metrics.total_transmitted,
        )
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_network_info(pool: &PgPool) -> Result<Vec<NetworkMetrics>, sqlx::Error> {
sqlx::query_as!(
        NetworkMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, total_received, total_transmitted 
        FROM network_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_network_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<NetworkMetrics, sqlx::Error> {
    sqlx::query_as!(
        NetworkMetrics,
        "SELECT sub_admin_metrics_id, staff_metrics_id, total_received, total_transmitted 
        FROM network_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}
