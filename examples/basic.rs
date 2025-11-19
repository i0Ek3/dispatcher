use dispatcher::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Dispatcher 基础示例\n");

    // 创建负载均衡策略
    let strategy = Arc::new(LoadBalancedStrategy::new());
    let engine = DispatchEngine::new(strategy).with_rules(vec![
        Box::new(CapacityRule),
        Box::new(DistanceRule::new(10.0)),
        Box::new(VehicleTypeRule),
    ]);

    // 添加配送员
    println!("📍 添加配送员...");
    let drivers = vec![
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
    ];

    for driver in drivers {
        println!(
            "  + {} ({})",
            driver.name,
            driver.vehicle_type.display_name()
        );
        engine.add_driver(driver);
    }

    // 创建并派发订单
    println!("\n📦 开始派单...\n");
    let mut orders = vec![
        Order::new(
            OrderType::Food,
            Location::new(39.9050, 116.4080),
            Location::new(39.9150, 116.4180),
        ),
        Order::new(
            OrderType::Express,
            Location::new(39.9020, 116.4050),
            Location::new(39.9120, 116.4150),
        ),
        Order::new(
            OrderType::RideHailing,
            Location::new(39.9080, 116.4120),
            Location::new(39.9500, 116.4500),
        ),
    ];

    for (i, order) in orders.iter_mut().enumerate() {
        println!("订单 #{}", i + 1);
        println!("  类型: {}", order.order_type.display_name());
        println!("  距离: {:.2} km", order.distance_km);
        println!("  预计时长: {} 分钟", order.estimated_duration_minutes);
        println!("  金额: ¥{:.2}", order.price);

        match engine.dispatch(order) {
            Ok(result) => {
                println!("  ✅ 已派单");
                println!("     配送员: {}", result.driver_name);
                println!("     车辆: {}", result.vehicle_type);
                println!("     距离取货点: {:.2} km", result.distance_to_pickup_km);
                println!("     预计到达: {} 分钟", result.estimated_arrival_minutes);
            }
            Err(e) => {
                println!("  ❌ 派单失败: {}", e);
            }
        }
        println!();
    }

    // 显示配送员状态
    println!("📊 配送员状态:");
    for driver in engine.get_all_drivers() {
        println!(
            "  {} - 负载: {}/{} ({:.0}%), 评分: {:.1}, 总单数: {}",
            driver.name,
            driver.current_load,
            driver.capacity,
            driver.load_ratio() * 100.0,
            driver.rating,
            driver.total_orders
        );
    }

    Ok(())
}
