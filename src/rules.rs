use super::*;

pub trait DispatchRule: Send + Sync {
    fn evaluate(&self, order: &Order, driver: &Driver) -> bool;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct CapacityRule;

impl DispatchRule for CapacityRule {
    fn evaluate(&self, _order: &Order, driver: &Driver) -> bool {
        driver.is_available()
    }

    fn name(&self) -> &str {
        "CapacityRule"
    }
}

#[derive(Debug)]
pub struct DistanceRule {
    pub max_distance_km: f64,
}

impl DistanceRule {
    pub fn new(max_distance_km: f64) -> Self {
        Self { max_distance_km }
    }
}

impl DispatchRule for DistanceRule {
    fn evaluate(&self, order: &Order, driver: &Driver) -> bool {
        driver.distance_to(order.pickup_location) <= self.max_distance_km
    }

    fn name(&self) -> &str {
        "DistanceRule"
    }
}

#[derive(Debug)]
pub struct VehicleTypeRule;

impl DispatchRule for VehicleTypeRule {
    fn evaluate(&self, order: &Order, driver: &Driver) -> bool {
        if order.order_type.requires_large_vehicle() {
            driver.vehicle_type.can_carry_large_items()
        } else {
            true
        }
    }

    fn name(&self) -> &str {
        "VehicleTypeRule"
    }
}

#[derive(Debug)]
pub struct RatingRule {
    pub min_rating: f64,
}

impl RatingRule {
    pub fn new(min_rating: f64) -> Self {
        Self { min_rating }
    }
}

impl DispatchRule for RatingRule {
    fn evaluate(&self, order: &Order, driver: &Driver) -> bool {
        if order.order_type.is_time_sensitive() {
            driver.rating >= self.min_rating
        } else {
            true
        }
    }

    fn name(&self) -> &str {
        "RatingRule"
    }
}
