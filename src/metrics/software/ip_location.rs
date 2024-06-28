use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use tokio::time::{interval, Duration};

#[derive(Serialize, Clone)]
pub struct IpLocation {
    pub sub_admin_metrics_id: Option<i32>,
    pub staff_metrics_id: Option<i32>,
    pub ip: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub isp: Option<String>,
}

impl IpLocation {
    pub fn new(
        sub_admin_metrics_id: Option<i32>,
        staff_metrics_id: Option<i32>,
        ip: Option<String>,
        city: Option<String>,
        region: Option<String>,
        country: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        isp: Option<String>,
    ) -> Self {
        Self {
            sub_admin_metrics_id,
            staff_metrics_id,
            ip,
            city,
            region,
            country,
            latitude,
            longitude,
            isp,
        }
    }

    pub fn update(
        &mut self,
        new_ip: Option<String>,
        new_city: Option<String>,
        new_region: Option<String>,
        new_country: Option<String>,
        new_latitude: Option<f64>,
        new_longitude: Option<f64>,
        new_isp: Option<String>,
    ) {
        self.ip = new_ip;
        self.city = new_city;
        self.region = new_region;
        self.country = new_country;
        self.latitude = new_latitude;
        self.longitude = new_longitude;
        self.isp = new_isp;
    }
}

async fn fetch_ip_location() -> Result<IpLocation, Box<dyn std::error::Error>> {
    let ip_response = reqwest::get("https://api.ipify.org").await?.text().await?;
    let ip_str = ip_response.trim().to_string();

    let url = format!("http://ip-api.com/json/{}", ip_str);
    let response: serde_json::Value = reqwest::get(&url).await?.json().await?;

    Ok(IpLocation {
        sub_admin_metrics_id: None,
        staff_metrics_id: None,
        ip: Some(ip_str),
        city: response.get("city").and_then(|v| v.as_str()).map(String::from),
        region: response.get("regionName").and_then(|v| v.as_str()).map(String::from),
        country: response.get("country").and_then(|v| v.as_str()).map(String::from),
        latitude: response.get("lat").and_then(|v| v.as_f64()),
        longitude: response.get("lon").and_then(|v| v.as_f64()),
        isp: response.get("isp").and_then(|v| v.as_str()).map(String::from),
    })
}

pub async fn gather_ip_location(
    sub_admin_metrics_id: Option<i32>,
    staff_metrics_id: Option<i32>,
) -> IpLocation {
    let mut interval = interval(Duration::from_secs(3600)); // Update every hour

    loop {
        interval.tick().await;
    let ip = match fetch_ip_location().await {
        Ok(location) => location,
        Err(e) => {
            eprintln!("Failed to fetch IP location: {}", e);
            IpLocation::new(sub_admin_metrics_id, staff_metrics_id, None, None, None, None, None, None, None)
        }
    };
    return ip
}
}

pub async fn get_ip_location_info_handler(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    let user_id_value = user_id.into_inner();

    match get_ip_location_info_for_user(&pool, user_id_value).await {
        Ok(ip_location_metrics) => {
            HttpResponse::Ok().json(ip_location_metrics)
        }
        Err(_) => {
            let new_ip_location_metrics = gather_ip_location(Some(user_id_value), None).await;
            if let Err(e) = save_ip_location_to_database(&pool, &new_ip_location_metrics).await {
                eprintln!("Failed to save metrics to database: {}", e);
                return HttpResponse::InternalServerError().body("Failed to save IP info");
            }
            HttpResponse::Ok().json(new_ip_location_metrics)
        }
    }
}

pub async fn save_ip_location_to_database(pool: &PgPool, metrics: &IpLocation) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query!(
        "INSERT INTO ip_location_metrics (sub_admin_metrics_id, staff_metrics_id, ip, city, region, country, latitude, longitude, isp) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        metrics.sub_admin_metrics_id,
        metrics.staff_metrics_id,
        metrics.ip,
        metrics.city,
        metrics.region,
        metrics.country,
        metrics.latitude,
        metrics.longitude,
        metrics.isp,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_ip_location_info(pool: &PgPool) -> Result<Vec<IpLocation>, sqlx::Error> {
    sqlx::query_as!(
        IpLocation,
        "SELECT sub_admin_metrics_id, staff_metrics_id, ip, city, region, country, latitude, longitude, isp FROM ip_location_metrics"
    )
    .fetch_all(pool)
    .await
}

async fn get_ip_location_info_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<IpLocation, sqlx::Error> {
    sqlx::query_as!(
        IpLocation,
        "SELECT sub_admin_metrics_id, staff_metrics_id, ip, city, region, country, latitude, longitude, isp 
        FROM ip_location_metrics 
        WHERE sub_admin_metrics_id = $1 OR staff_metrics_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
}
