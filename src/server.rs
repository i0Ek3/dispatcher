use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use dispatcher::*;
use serde::{Deserialize, Serialize};

// ============ API 请求/响应结构 ============

#[derive(Debug, Serialize, Deserialize)]
struct DispatchRequest {
    order_type: String,
    pickup_lat: f64,
    pickup_lng: f64,
    delivery_lat: f64,
    delivery_lng: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DispatchResponse {
    success: bool,
    message: String,
    data: Option<DispatchResultData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DispatchResultData {
    order_id: String,
    driver_id: String,
    driver_name: String,
    vehicle_type: String,
    distance_to_pickup_km: f64,
    estimated_arrival_minutes: u32,
    order_distance_km: f64,
    order_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriverInfo {
    id: String,
    name: String,
    status: String,
    vehicle_type: String,
    current_location: LocationData,
    capacity: usize,
    current_load: usize,
    rating: f64,
    total_orders: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocationData {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddDriverRequest {
    name: String,
    vehicle_type: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateLocationRequest {
    driver_id: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StrategyRequest {
    strategy: String,
}

// ============ 应用状态 ============

struct AppState {
    engine: Arc<Mutex<DispatchEngine>>,
    current_strategy: Arc<Mutex<String>>,
}

// ============ API 路由处理器 ============

/// 派单接口
async fn dispatch_order(
    data: web::Data<AppState>,
    req: web::Json<DispatchRequest>,
) -> HttpResponse {
    let order_type = match req.order_type.as_str() {
        "外卖订单" | "Food" => OrderType::Food,
        "快递订单" | "Express" => OrderType::Express,
        "生鲜订单" | "FreshFood" => OrderType::FreshFood,
        "打车订单" | "RideHailing" => OrderType::RideHailing,
        "货运订单" | "Cargo" => OrderType::Cargo,
        _ => OrderType::Food,
    };

    let pickup = Location::new(req.pickup_lat, req.pickup_lng);
    let delivery = Location::new(req.delivery_lat, req.delivery_lng);

    let mut order = Order::new(order_type, pickup, delivery);

    let engine = data.engine.lock().unwrap();
    match engine.dispatch(&mut order) {
        Ok(result) => {
            let response = DispatchResponse {
                success: true,
                message: "派单成功".to_string(),
                data: Some(DispatchResultData {
                    order_id: result.order_id.to_string(),
                    driver_id: result.driver_id.to_string(),
                    driver_name: result.driver_name.clone(),
                    vehicle_type: result.vehicle_type.clone(),
                    distance_to_pickup_km: result.distance_to_pickup_km,
                    estimated_arrival_minutes: result.estimated_arrival_minutes,
                    order_distance_km: order.distance_km,
                    order_price: order.price,
                }),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response = DispatchResponse {
                success: false,
                message: format!("派单失败: {}", e),
                data: None,
            };
            HttpResponse::Ok().json(response)
        }
    }
}

/// 获取所有配送员信息
async fn get_drivers(data: web::Data<AppState>) -> HttpResponse {
    let engine = data.engine.lock().unwrap();
    let drivers = engine.get_all_drivers();

    let driver_infos: Vec<DriverInfo> = drivers
        .iter()
        .map(|d| DriverInfo {
            id: d.id.to_string(),
            name: d.name.clone(),
            status: format!("{:?}", d.status),
            vehicle_type: d.vehicle_type.display_name().to_string(),
            current_location: LocationData {
                latitude: d.current_location.latitude,
                longitude: d.current_location.longitude,
            },
            capacity: d.capacity,
            current_load: d.current_load,
            rating: d.rating,
            total_orders: d.total_orders,
        })
        .collect();

    HttpResponse::Ok().json(driver_infos)
}

/// 添加配送员
async fn add_driver(
    data: web::Data<AppState>,
    req: web::Json<AddDriverRequest>,
) -> HttpResponse {
    let vehicle_type = match req.vehicle_type.as_str() {
        "电动车" | "ElectricBike" => VehicleType::ElectricBike,
        "摩托车" | "Motorcycle" => VehicleType::Motorcycle,
        "汽车" | "Car" => VehicleType::Car,
        "面包车" | "Van" => VehicleType::Van,
        "卡车" | "Truck" => VehicleType::Truck,
        _ => VehicleType::ElectricBike,
    };

    let location = Location::new(req.latitude, req.longitude);
    let driver = Driver::new(req.name.clone(), vehicle_type, location);
    let driver_id = driver.id.to_string();

    let engine = data.engine.lock().unwrap();
    engine.add_driver(driver);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "配送员添加成功",
        "driver_id": driver_id
    }))
}

/// 更新配送员位置
async fn update_driver_location(
    data: web::Data<AppState>,
    req: web::Json<UpdateLocationRequest>,
) -> HttpResponse {
    let driver_id = match uuid::Uuid::parse_str(&req.driver_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的配送员ID"
            }))
        }
    };

    let new_location = Location::new(req.latitude, req.longitude);
    let engine = data.engine.lock().unwrap();

    match engine.update_driver_location(&driver_id, new_location) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "位置更新成功"
        })),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "message": format!("位置更新失败: {}", e)
        })),
    }
}

/// 释放订单（配送员完成订单）
async fn release_order(
    data: web::Data<AppState>,
    driver_id: web::Path<String>,
) -> HttpResponse {
    let driver_id = match uuid::Uuid::parse_str(&driver_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "无效的配送员ID"
            }))
        }
    };

    let engine = data.engine.lock().unwrap();
    match engine.release_order(&driver_id) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "订单释放成功"
        })),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "message": format!("订单释放失败: {}", e)
        })),
    }
}

/// 切换派单策略
async fn change_strategy(
    data: web::Data<AppState>,
    req: web::Json<StrategyRequest>,
) -> HttpResponse {
    let new_strategy: Arc<dyn DispatchStrategy> = match req.strategy.as_str() {
        "nearest_first" => Arc::new(NearestFirstStrategy::new()),
        "load_balanced" => Arc::new(LoadBalancedStrategy::new()),
        "rating_priority" => Arc::new(RatingPriorityStrategy::new()),
        "capacity_match" => Arc::new(CapacityMatchStrategy::new()),
        _ => Arc::new(NearestFirstStrategy::new()),
    };

    // 创建新引擎（保留配送员）
    let old_engine = data.engine.lock().unwrap();
    let drivers = old_engine.get_all_drivers();
    drop(old_engine);

    let new_engine = DispatchEngine::new(new_strategy)
        .with_rules(vec![
            Box::new(CapacityRule),
            Box::new(DistanceRule::new(10.0)),
            Box::new(VehicleTypeRule),
            Box::new(RatingRule::new(4.0)),
        ]);

    for driver in drivers {
        new_engine.add_driver(driver);
    }

    let mut engine = data.engine.lock().unwrap();
    *engine = new_engine;

    let mut current_strategy = data.current_strategy.lock().unwrap();
    *current_strategy = req.strategy.clone();

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("策略已切换至: {}", req.strategy)
    }))
}

/// 健康检查
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Dispatcher API is running"
    }))
}

/// 获取当前策略
async fn get_current_strategy(data: web::Data<AppState>) -> HttpResponse {
    let strategy = data.current_strategy.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "strategy": *strategy
    }))
}

// ============ 主函数 ============

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 启动 Dispatcher Web Server...\n");

    // 初始化引擎
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy)
        .with_rules(vec![
            Box::new(CapacityRule),
            Box::new(DistanceRule::new(10.0)),
            Box::new(VehicleTypeRule),
            Box::new(RatingRule::new(4.0)),
        ]);

    // 添加初始配送员
    println!("📍 初始化配送员...");
    let initial_drivers = vec![
        Driver::new(
            "张师傅".to_string(),
            VehicleType::ElectricBike,
            Location::new(39.9042, 116.4074),
        ),
        Driver::new(
            "李师傅".to_string(),
            VehicleType::Motorcycle,
            Location::new(39.9100, 116.4100),
        ),
        Driver::new(
            "王师傅".to_string(),
            VehicleType::Car,
            Location::new(39.9000, 116.4000),
        ),
        Driver::new(
            "赵师傅".to_string(),
            VehicleType::ElectricBike,
            Location::new(39.9080, 116.4120),
        ),
    ];

    for driver in initial_drivers {
        println!("  + {} ({})", driver.name, driver.vehicle_type.display_name());
        engine.add_driver(driver);
    }

    // 创建应用状态
    let app_state = web::Data::new(AppState {
        engine: Arc::new(Mutex::new(engine)),
        current_strategy: Arc::new(Mutex::new("nearest_first".to_string())),
    });

    println!("\n✅ 服务器启动成功!");
    println!("📡 API 地址: http://127.0.0.1:8080");
    println!("🌐 Web 界面: 请将 web/app.js 中的 API 地址设置为 http://127.0.0.1:8080\n");
    println!("API 端点:");
    println!("  POST   /api/dispatch          - 派单");
    println!("  GET    /api/drivers           - 获取配送员列表");
    println!("  POST   /api/drivers           - 添加配送员");
    println!("  PUT    /api/drivers/location  - 更新配送员位置");
    println!("  POST   /api/drivers/:id/release - 释放订单");
    println!("  POST   /api/strategy          - 切换策略");
    println!("  GET    /api/strategy          - 获取当前策略");
    println!("  GET    /health                - 健康检查\n");

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        // 配置 CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // API 路由
            .route("/health", web::get().to(health_check))
            .route("/api/dispatch", web::post().to(dispatch_order))
            .route("/api/drivers", web::get().to(get_drivers))
            .route("/api/drivers", web::post().to(add_driver))
            .route("/api/drivers/location", web::put().to(update_driver_location))
            .route("/api/drivers/{id}/release", web::post().to(release_order))
            .route("/api/strategy", web::post().to(change_strategy))
            .route("/api/strategy", web::get().to(get_current_strategy))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}