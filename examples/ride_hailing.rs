use dispatcher::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚗 网约车系统示例\n");
    println!("场景：城市晚高峰打车场景\n");

    // 使用高评分优先策略
    let strategy = Arc::new(RatingPriorityStrategy::new());
    let engine = DispatchEngine::new(strategy).with_rules(vec![
        Box::new(CapacityRule),
        Box::new(DistanceRule::new(10.0)), // 接驾最远10公里
        Box::new(RatingRule::new(4.5)),
    ]);

    // 添加司机
    println!("📍 初始化司机团队...");
    let mut drivers = vec![
        Driver::new(
            "赵师傅".to_string(),
            VehicleType::Car,
            Location::new(39.9042, 116.4074),
        ),
        Driver::new(
            "钱师傅".to_string(),
            VehicleType::Car,
            Location::new(39.9100, 116.4100),
        ),
        Driver::new(
            "孙师傅".to_string(),
            VehicleType::Van,
            Location::new(39.9000, 116.4000),
        ),
    ];

    for (i, driver) in drivers.iter_mut().enumerate() {
        driver.rating = 4.6 + (i as f64 * 0.15);
        driver.total_orders = 100 + (i * 50) as u32;
        println!(
            "  {} - {} - 评分 {:.1} - 完成 {} 单",
            driver.name,
            driver.vehicle_type.display_name(),
            driver.rating,
            driver.total_orders
        );
        engine.add_driver(driver.clone());
    }

    // 模拟打车订单
    println!("\n🚖 乘客呼叫...\n");

    let rides = vec![
        (
            "王先生 - 从国贸到首都机场",
            Location::new(39.9088, 116.3974),
            Location::new(40.0798, 116.6031),
        ),
        (
            "李女士 - 从三里屯到西单",
            Location::new(39.9368, 116.4472),
            Location::new(39.9091, 116.3745),
        ),
        (
            "张先生 - 从中关村到望京",
            Location::new(39.9827, 116.3089),
            Location::new(39.9952, 116.4733),
        ),
    ];

    for (i, (passenger, pickup, delivery)) in rides.iter().enumerate() {
        let mut order = Order::new(OrderType::RideHailing, *pickup, *delivery);

        println!("订单 #{} - {}", i + 1, passenger);
        println!("  行程距离: {:.2} km", order.distance_km);
        println!("  预计时长: {} 分钟", order.estimated_duration_minutes);
        println!("  预估费用: ¥{:.2}", order.price);

        match engine.dispatch(&mut order) {
            Ok(result) => {
                println!("  ✅ 司机接单");
                println!(
                    "     司机: {} (评分 {:.1})",
                    result.driver_name,
                    engine.get_driver_stats(&result.driver_id)?.rating
                );
                println!("     车型: {}", result.vehicle_type);
                println!("     接驾距离: {:.2} km", result.distance_to_pickup_km);
                println!("     预计 {} 分钟到达", result.estimated_arrival_minutes);
            }
            Err(e) => {
                println!("  ❌ 派单失败: {}", e);
            }
        }
        println!();
    }

    println!("📊 司机状态:");
    for driver in engine.get_all_drivers() {
        println!(
            "  {} - 正在服务: {} 单, 总计完成: {} 单, 评分: {:.1}",
            driver.name, driver.current_load, driver.total_orders, driver.rating
        );
    }

    Ok(())
}
