use super::*;

pub trait DispatchStrategy: Send + Sync {
    fn select_driver<'a>(
        &self,
        order: &Order,
        drivers: Vec<&'a Driver>,
        rules: &[Box<dyn DispatchRule>],
    ) -> Option<&'a Driver>;

    fn name(&self) -> &str;
}

/// 就近派单策略：选择距离订单取货点最近的司机
#[derive(Debug)]
pub struct NearestFirstStrategy;

impl NearestFirstStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NearestFirstStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchStrategy for NearestFirstStrategy {
    fn select_driver<'a>(
        &self,
        order: &Order,
        drivers: Vec<&'a Driver>,
        rules: &[Box<dyn DispatchRule>],
    ) -> Option<&'a Driver> {
        drivers
            .into_iter()
            .filter(|driver| rules.iter().all(|rule| rule.evaluate(order, driver)))
            .min_by(|a, b| {
                let dist_a = a.distance_to(order.pickup_location);
                let dist_b = b.distance_to(order.pickup_location);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn name(&self) -> &str {
        "NearestFirst"
    }
}

/// 负载均衡策略：优先分配给负载率最低的司机
#[derive(Debug)]
pub struct LoadBalancedStrategy;

impl LoadBalancedStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoadBalancedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchStrategy for LoadBalancedStrategy {
    fn select_driver<'a>(
        &self,
        order: &Order,
        drivers: Vec<&'a Driver>,
        rules: &[Box<dyn DispatchRule>],
    ) -> Option<&'a Driver> {
        drivers
            .into_iter()
            .filter(|driver| rules.iter().all(|rule| rule.evaluate(order, driver)))
            .min_by(|a, b| {
                a.load_ratio()
                    .partial_cmp(&b.load_ratio())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn name(&self) -> &str {
        "LoadBalanced"
    }
}

/// 高评分优先策略：优先分配给评分高的司机
#[derive(Debug)]
pub struct RatingPriorityStrategy;

impl RatingPriorityStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RatingPriorityStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchStrategy for RatingPriorityStrategy {
    fn select_driver<'a>(
        &self,
        order: &Order,
        drivers: Vec<&'a Driver>,
        rules: &[Box<dyn DispatchRule>],
    ) -> Option<&'a Driver> {
        let mut eligible: Vec<_> = drivers
            .into_iter()
            .filter(|driver| rules.iter().all(|rule| rule.evaluate(order, driver)))
            .collect();

        eligible.sort_by(|a, b| {
            b.rating
                .partial_cmp(&a.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.load_ratio()
                        .partial_cmp(&b.load_ratio())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        eligible.first().copied()
    }

    fn name(&self) -> &str {
        "RatingPriority"
    }
}

/// 运力匹配策略：根据订单类型选择合适的车辆
#[derive(Debug)]
pub struct CapacityMatchStrategy;

impl CapacityMatchStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CapacityMatchStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchStrategy for CapacityMatchStrategy {
    fn select_driver<'a>(
        &self,
        order: &Order,
        drivers: Vec<&'a Driver>,
        rules: &[Box<dyn DispatchRule>],
    ) -> Option<&'a Driver> {
        let eligible: Vec<_> = drivers
            .into_iter()
            .filter(|driver| rules.iter().all(|rule| rule.evaluate(order, driver)))
            .collect();

        // 如果需要大型车辆，优先选择大车
        let preferred = if order.order_type.requires_large_vehicle() {
            eligible
                .iter()
                .filter(|d| d.vehicle_type.can_carry_large_items())
                .copied()
                .collect::<Vec<_>>()
        } else {
            eligible.clone()
        };

        let candidates = if preferred.is_empty() {
            eligible
        } else {
            preferred
        };

        // 在符合条件的司机中选择距离最近的
        candidates.into_iter().min_by(|a, b| {
            let dist_a = a.distance_to(order.pickup_location);
            let dist_b = b.distance_to(order.pickup_location);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn name(&self) -> &str {
        "CapacityMatch"
    }
}
