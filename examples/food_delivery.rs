use dispatcher::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍔 外卖配送系统示例\n");
    println!("场景：某外卖平台午高峰时段的订单派送\n");

    // 使用就近派单策略
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy).with_rules(vec![
        Box::new(CapacityRule),
        Box::new(DistanceRule::new(5.0)), // 外卖最远5公里
        Box::new(RatingRule::new(4.5)),   // 最低评分4.5
    ]);

    // 添加骑手
    println!("📍 初始化骑手团队...");
    let mut riders = vec![
        Driver::new(
            "张骑手".to_string(),
            VehicleType::ElectricBike,
            Location::new(39.9042, 116.4074),
        ),
        Driver::new(
            "李骑手".to_string(),
            VehicleType::Motorcycle,
            Location::new(39.9100, 116.4100),
        ),
        Driver::new(
            "王骑手".to_string(),
            VehicleType::ElectricBike,
            Location::new(39.9000, 116.4000),
        ),
        Driver::new(
            "赵骑手".to_string(),
            VehicleType::Motorcycle,
            Location::new(39.9080, 116.4120),
        ),
    ];

    // 设置骑手评分
    for (i, rider) in riders.iter_mut().enumerate() {
        rider.rating = 4.5 + (i as f64 * 0.1);
        println!(
            "  {} - {} - 评分 {:.1}",
            rider.name,
            rider.vehicle_type.display_name(),
            rider.rating
        );
        engine.add_driver(rider.clone());
    }

    // 模拟午高峰订单
    println!("\n🍜 午高峰订单涌入...\n");
    let restaurants = vec![
        ("麦当劳", Location::new(39.9050, 116.4080)),
        ("肯德基", Location::new(39.9030, 116.4060)),
        ("必胜客", Location::new(39.9070, 116.4110)),
        ("星巴克", Location::new(39.9010, 116.4040)),
        ("海底捞", Location::new(39.9090, 116.4130)),
    ];

    let mut success_count = 0;
    let mut total_distance = 0.0;

    for (i, (restaurant, pickup_loc)) in restaurants.iter().enumerate() {
        let delivery_loc = Location::new(pickup_loc.latitude + 0.01, pickup_loc.longitude + 0.01);

        let mut order = Order::new(OrderType::Food, *pickup_loc, delivery_loc);

        println!("订单 #{} - {}", i + 1, restaurant);
        println!(
            "  取餐地址: ({:.4}, {:.4})",
            pickup_loc.latitude, pickup_loc.longitude
        );
        println!("  送餐距离: {:.2} km", order.distance_km);
        println!("  订单金额: ¥{:.2}", order.price);

        match engine.dispatch(&mut order) {
            Ok(result) => {
                success_count += 1;
                total_distance += result.distance_to_pickup_km;
                println!("  ✅ 派给: {}", result.driver_name);
                println!("  距离商家: {:.2} km", result.distance_to_pickup_km);
                println!("  预计 {} 分钟送达", order.estimated_duration_minutes);
            }
            Err(e) => {
                println!("  ❌ 派单失败: {}", e);
            }
        }
        println!();
    }

    // 统计信息
    println!("📈 派单统计:");
    println!("  总订单数: {}", restaurants.len());
    println!("  成功派单: {}", success_count);
    println!(
        "  派单成功率: {:.1}%",
        (success_count as f64 / restaurants.len() as f64) * 100.0
    );
    println!(
        "  平均接单距离: {:.2} km",
        total_distance / success_count as f64
    );

    println!("\n👥 骑手状态:");
    for driver in engine.get_all_drivers() {
        println!(
            "  {} - 当前接单: {} 单, 负载率: {:.0}%",
            driver.name,
            driver.current_load,
            driver.load_ratio() * 100.0
        );
    }

    Ok(())
}
