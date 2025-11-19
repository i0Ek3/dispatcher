use dispatcher::*;
use std::io::{self, Write};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════╗");
    println!("       Dispatcher 配送分单引擎演示系统       ");
    println!("╚════════════════════════════════════════╝\n");

    loop {
        println!("\n请选择演示场景:");
        println!("  1. 基础派单演示");
        println!("  2. 外卖配送场景");
        println!("  3. 网约车场景");
        println!("  4. 策略对比演示");
        println!("  0. 退出");
        print!("\n请输入选项: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => demo_basic()?,
            "2" => demo_food_delivery()?,
            "3" => demo_ride_hailing()?,
            "4" => demo_strategy_comparison()?,
            "0" => {
                println!("\n👋 感谢使用！");
                break;
            }
            _ => println!("❌ 无效选项，请重新选择"),
        }
    }

    Ok(())
}

fn demo_basic() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n".repeat(2));
    println!("════════════════════════════════════════");
    println!("         基础派单演示");
    println!("════════════════════════════════════════\n");

    let strategy = Arc::new(LoadBalancedStrategy::new());
    let engine = DispatchEngine::new(strategy);

    // 添加司机
    engine.add_driver(Driver::new(
        "张师傅".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    ));
    engine.add_driver(Driver::new(
        "李师傅".to_string(),
        VehicleType::Car,
        Location::new(39.9100, 116.4100),
    ));

    println!("✅ 已添加 2 名配送员\n");

    // 派单
    let mut order = Order::new(
        OrderType::Food,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );

    println!("📦 新订单: {}", order.order_type.display_name());
    println!("   距离: {:.2} km", order.distance_km);
    println!("   金额: ¥{:.2}\n", order.price);

    match engine.dispatch(&mut order) {
        Ok(result) => {
            println!("✅ 派单成功!");
            println!("   配送员: {}", result.driver_name);
            println!("   车型: {}", result.vehicle_type);
        }
        Err(e) => println!("❌ 派单失败: {}", e),
    }

    Ok(())
}

fn demo_food_delivery() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n".repeat(2));
    println!("════════════════════════════════════════");
    println!("         外卖配送场景");
    println!("════════════════════════════════════════\n");

    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy).with_rules(vec![
        Box::new(CapacityRule),
        Box::new(DistanceRule::new(5.0)),
    ]);

    // 添加骑手
    for i in 1..=3 {
        let driver = Driver::new(
            format!("骑手{}", i),
            VehicleType::ElectricBike,
            Location::new(39.9042 + i as f64 * 0.005, 116.4074),
        );
        engine.add_driver(driver);
    }

    println!("✅ 已添加 3 名骑手\n");

    // 模拟5个外卖订单
    for i in 1..=5 {
        let mut order = Order::new(
            OrderType::Food,
            Location::new(39.9050 + i as f64 * 0.003, 116.4080),
            Location::new(39.9150 + i as f64 * 0.003, 116.4180),
        );

        println!("订单 #{}: 距离 {:.2}km", i, order.distance_km);
        match engine.dispatch(&mut order) {
            Ok(result) => println!("  ✅ 派给: {}", result.driver_name),
            Err(e) => println!("  ❌ 失败: {}", e),
        }
    }

    Ok(())
}

fn demo_ride_hailing() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n".repeat(2));
    println!("════════════════════════════════════════");
    println!("         网约车场景");
    println!("════════════════════════════════════════\n");

    let strategy = Arc::new(RatingPriorityStrategy::new());
    let engine = DispatchEngine::new(strategy);

    // 添加司机
    let mut driver1 = Driver::new(
        "高分司机".to_string(),
        VehicleType::Car,
        Location::new(39.9042, 116.4074),
    );
    driver1.rating = 4.9;

    let mut driver2 = Driver::new(
        "普通司机".to_string(),
        VehicleType::Car,
        Location::new(39.9042, 116.4074),
    );
    driver2.rating = 4.6;

    engine.add_driver(driver1);
    engine.add_driver(driver2);

    println!("✅ 已添加 2 名司机\n");

    // 打车订单
    let mut order = Order::new(
        OrderType::RideHailing,
        Location::new(39.9050, 116.4080),
        Location::new(39.9500, 116.4500),
    );

    println!("🚖 乘客呼叫: 距离 {:.2}km", order.distance_km);
    match engine.dispatch(&mut order) {
        Ok(result) => {
            println!("✅ 司机接单: {}", result.driver_name);
            let driver = engine.get_driver_stats(&result.driver_id)?;
            println!("   评分: {:.1}", driver.rating);
        }
        Err(e) => println!("❌ 失败: {}", e),
    }

    Ok(())
}

fn demo_strategy_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "\n".repeat(2));
    println!("════════════════════════════════════════");
    println!("         策略对比演示");
    println!("════════════════════════════════════════\n");

    let strategies: Vec<(&str, Arc<dyn DispatchStrategy>)> = vec![
        ("就近派单", Arc::new(NearestFirstStrategy::new())),
        ("负载均衡", Arc::new(LoadBalancedStrategy::new())),
        ("高评分优先", Arc::new(RatingPriorityStrategy::new())),
        ("运力匹配", Arc::new(CapacityMatchStrategy::new())),
    ];

    // 准备测试数据
    let test_drivers = vec![
        (
            "张师傅",
            VehicleType::ElectricBike,
            Location::new(39.9042, 116.4074),
            4.5,
            1,
        ),
        (
            "李师傅",
            VehicleType::Car,
            Location::new(39.9100, 116.4100),
            4.9,
            0,
        ),
        (
            "王师傅",
            VehicleType::Motorcycle,
            Location::new(39.9000, 116.4000),
            4.7,
            2,
        ),
    ];

    let test_order = Order::new(
        OrderType::Food,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );

    println!("📦 测试订单:");
    println!("   类型: {}", test_order.order_type.display_name());
    println!("   距离: {:.2} km", test_order.distance_km);
    println!("   金额: ¥{:.2}\n", test_order.price);

    println!("👥 可用配送员:");
    for (name, vehicle, location, rating, load) in &test_drivers {
        let dist = distance_km(*location, test_order.pickup_location);
        println!(
            "   {} - {} - 评分{:.1} - 负载{} - 距离{:.2}km",
            name,
            vehicle.display_name(),
            rating,
            load,
            dist
        );
    }
    println!();

    // 对比不同策略
    for (strategy_name, strategy) in strategies {
        println!("策略: {}", strategy_name);

        let engine = DispatchEngine::new(strategy.clone());

        // 添加配送员
        for (name, vehicle, location, rating, load) in &test_drivers {
            let mut driver = Driver::new(name.to_string(), vehicle.clone(), *location);
            driver.rating = *rating;
            driver.current_load = *load;
            engine.add_driver(driver);
        }

        // 派单
        let mut order = test_order.clone();
        match engine.dispatch(&mut order) {
            Ok(result) => {
                println!("  ✅ 派给: {}", result.driver_name);
                println!("     距离: {:.2} km", result.distance_to_pickup_km);
                println!("     预计: {} 分钟到达", result.estimated_arrival_minutes);
            }
            Err(e) => {
                println!("  ❌ 派单失败: {}", e);
            }
        }
        println!();
    }

    println!("💡 策略分析:");
    println!("   • 就近派单: 选择距离最近的配送员，配送速度最快");
    println!("   • 负载均衡: 选择负载最低的配送员，分配更公平");
    println!("   • 高评分优先: 选择评分最高的配送员，服务质量最好");
    println!("   • 运力匹配: 根据订单类型选择合适车型，资源优化");

    Ok(())
}
