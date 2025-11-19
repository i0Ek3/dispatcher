use dispatcher::*;
use std::sync::Arc;

#[test]
fn test_nearest_first_strategy() {
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy);

    // 添加两个配送员，位置不同
    let driver1 = Driver::new(
        "近的司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    let driver2 = Driver::new(
        "远的司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9200, 116.4300),
    );

    engine.add_driver(driver1);
    engine.add_driver(driver2);

    // 创建订单，取餐点靠近driver1
    let mut order = Order::new(
        OrderType::Food,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );

    let result = engine.dispatch(&mut order);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().driver_name, "近的司机");
}

#[test]
fn test_load_balanced_strategy() {
    let strategy = Arc::new(LoadBalancedStrategy::new());
    let engine = DispatchEngine::new(strategy);

    let mut driver1 = Driver::new(
        "空闲司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    driver1.current_load = 0;

    let mut driver2 = Driver::new(
        "忙碌司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    driver2.current_load = 2;

    engine.add_driver(driver1);
    engine.add_driver(driver2);

    let mut order = Order::new(
        OrderType::Food,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );

    let result = engine.dispatch(&mut order);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().driver_name, "空闲司机");
}

#[test]
fn test_rating_priority_strategy() {
    let strategy = Arc::new(RatingPriorityStrategy::new());
    let engine = DispatchEngine::new(strategy);

    let mut driver1 = Driver::new(
        "低分司机".to_string(),
        VehicleType::Car,
        Location::new(39.9042, 116.4074),
    );
    driver1.rating = 4.5;

    let mut driver2 = Driver::new(
        "高分司机".to_string(),
        VehicleType::Car,
        Location::new(39.9042, 116.4074),
    );
    driver2.rating = 4.9;

    engine.add_driver(driver1);
    engine.add_driver(driver2);

    let mut order = Order::new(
        OrderType::RideHailing,
        Location::new(39.9050, 116.4080),
        Location::new(39.9500, 116.4500),
    );

    let result = engine.dispatch(&mut order);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().driver_name, "高分司机");
}

#[test]
fn test_distance_rule() {
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy).with_rules(vec![
        Box::new(CapacityRule),
        Box::new(DistanceRule::new(1.0)), // 只接1公里内的单
    ]);

    let driver = Driver::new(
        "司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    engine.add_driver(driver);

    // 订单距离超过1公里
    let mut order = Order::new(
        OrderType::Food,
        Location::new(39.9200, 116.4300),
        Location::new(39.9250, 116.4350),
    );

    let result = engine.dispatch(&mut order);
    assert!(result.is_err());
}

#[test]
fn test_vehicle_type_rule() {
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy)
        .with_rules(vec![Box::new(CapacityRule), Box::new(VehicleTypeRule)]);

    // 只有电动车
    let driver = Driver::new(
        "骑手".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    engine.add_driver(driver);

    // 快递订单需要大车
    let mut order = Order::new(
        OrderType::Express,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );

    let result = engine.dispatch(&mut order);
    assert!(result.is_err()); // 应该失败，因为电动车不能接快递
}

#[test]
fn test_capacity_exceeded() {
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy);

    let mut driver = Driver::new(
        "司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    driver.capacity = 1;
    driver.current_load = 1; // 已满载
    driver.status = DriverStatus::Busy;

    engine.add_driver(driver);

    let mut order = Order::new(
        OrderType::Food,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );

    let result = engine.dispatch(&mut order);
    assert!(result.is_err());
}

#[test]
fn test_distance_calculation() {
    let loc1 = Location::new(39.9042, 116.4074); // 天安门
    let loc2 = Location::new(40.0798, 116.6031); // 首都机场

    let distance = distance_km(loc1, loc2);
    assert!(distance > 20.0 && distance < 30.0); // 实际距离约26公里
}

#[test]
fn test_order_price_calculation() {
    let order = Order::new(
        OrderType::Food,
        Location::new(39.9042, 116.4074),
        Location::new(39.9150, 116.4180),
    );

    assert!(order.price > 0.0);
    assert!(order.estimated_duration_minutes > 0);
}

#[test]
fn test_driver_release() {
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy);

    let driver = Driver::new(
        "司机".to_string(),
        VehicleType::ElectricBike,
        Location::new(39.9042, 116.4074),
    );
    let driver_id = driver.id;
    engine.add_driver(driver);

    // 派单
    let mut order = Order::new(
        OrderType::Food,
        Location::new(39.9050, 116.4080),
        Location::new(39.9150, 116.4180),
    );
    engine.dispatch(&mut order).unwrap();

    // 检查负载
    let driver_stats = engine.get_driver_stats(&driver_id).unwrap();
    assert_eq!(driver_stats.current_load, 1);

    // 释放订单
    engine.release_order(&driver_id).unwrap();
    let driver_stats = engine.get_driver_stats(&driver_id).unwrap();
    assert_eq!(driver_stats.current_load, 0);
}

#[test]
fn test_update_driver_location() {
    let strategy = Arc::new(NearestFirstStrategy::new());
    let engine = DispatchEngine::new(strategy);

    let driver = Driver::new(
        "司机".to_string(),
        VehicleType::Car,
        Location::new(39.9042, 116.4074),
    );
    let driver_id = driver.id;
    engine.add_driver(driver);

    // 更新位置
    let new_location = Location::new(39.9100, 116.4100);
    engine
        .update_driver_location(&driver_id, new_location)
        .unwrap();

    let driver_stats = engine.get_driver_stats(&driver_id).unwrap();
    assert_eq!(driver_stats.current_location.latitude, 39.9100);
    assert_eq!(driver_stats.current_location.longitude, 116.4100);
}
