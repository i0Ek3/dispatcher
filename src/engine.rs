use super::*;
use std::sync::{Arc, RwLock};

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("No available driver found")]
    NoDriverAvailable,

    #[error("Driver not found: {0}")]
    DriverNotFound(Uuid),

    #[error("Order not found: {0}")]
    OrderNotFound(Uuid),

    #[error("Driver capacity exceeded")]
    CapacityExceeded,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    pub order_id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub vehicle_type: String,
    pub distance_to_pickup_km: f64,
    pub estimated_arrival_minutes: u32,
    pub strategy_used: String,
    pub timestamp: i64,
}

pub struct DispatchEngine {
    driver_pool: Arc<RwLock<DriverPool>>,
    strategy: Arc<dyn DispatchStrategy>,
    rules: Vec<Box<dyn DispatchRule>>,
}

impl DispatchEngine {
    pub fn new(strategy: Arc<dyn DispatchStrategy>) -> Self {
        Self {
            driver_pool: Arc::new(RwLock::new(DriverPool::new())),
            strategy,
            rules: vec![Box::new(CapacityRule), Box::new(VehicleTypeRule)],
        }
    }

    pub fn with_rules(mut self, rules: Vec<Box<dyn DispatchRule>>) -> Self {
        self.rules = rules;
        self
    }

    pub fn add_rule(&mut self, rule: Box<dyn DispatchRule>) {
        self.rules.push(rule);
    }

    pub fn add_driver(&self, driver: Driver) {
        let mut pool = self.driver_pool.write().unwrap();
        pool.add_driver(driver);
    }

    pub fn remove_driver(&self, driver_id: &Uuid) -> Result<Driver, DispatchError> {
        let mut pool = self.driver_pool.write().unwrap();
        pool.remove_driver(driver_id)
            .ok_or(DispatchError::DriverNotFound(*driver_id))
    }

    pub fn dispatch(&self, order: &mut Order) -> Result<DispatchResult, DispatchError> {
        let pool = self.driver_pool.read().unwrap();
        let available_drivers = pool.get_available_drivers();

        let selected_driver = self
            .strategy
            .select_driver(order, available_drivers, &self.rules)
            .ok_or(DispatchError::NoDriverAvailable)?;

        let driver_id = selected_driver.id;
        let driver_name = selected_driver.name.clone();
        let vehicle_type = selected_driver.vehicle_type.display_name().to_string();
        let distance_to_pickup_km = selected_driver.distance_to(order.pickup_location);
        let estimated_arrival_minutes = (distance_to_pickup_km / 30.0 * 60.0) as u32; // 假设30km/h

        drop(pool);

        let mut pool = self.driver_pool.write().unwrap();
        let driver = pool
            .get_driver_mut(&driver_id)
            .ok_or(DispatchError::DriverNotFound(driver_id))?;

        if !driver.assign_order() {
            return Err(DispatchError::CapacityExceeded);
        }

        order.status = OrderStatus::Dispatched;
        order.assigned_to = Some(driver_id);

        Ok(DispatchResult {
            order_id: order.id,
            driver_id,
            driver_name,
            vehicle_type,
            distance_to_pickup_km,
            estimated_arrival_minutes,
            strategy_used: self.strategy.name().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    pub fn release_order(&self, driver_id: &Uuid) -> Result<(), DispatchError> {
        let mut pool = self.driver_pool.write().unwrap();
        let driver = pool
            .get_driver_mut(driver_id)
            .ok_or(DispatchError::DriverNotFound(*driver_id))?;

        driver.release_order();
        Ok(())
    }

    pub fn get_driver_stats(&self, driver_id: &Uuid) -> Result<Driver, DispatchError> {
        let pool = self.driver_pool.read().unwrap();
        pool.get_driver(driver_id)
            .cloned()
            .ok_or(DispatchError::DriverNotFound(*driver_id))
    }

    pub fn get_all_drivers(&self) -> Vec<Driver> {
        let pool = self.driver_pool.read().unwrap();
        pool.get_all_drivers().into_iter().cloned().collect()
    }

    pub fn update_driver_location(
        &self,
        driver_id: &Uuid,
        new_location: Location,
    ) -> Result<(), DispatchError> {
        let mut pool = self.driver_pool.write().unwrap();
        let driver = pool
            .get_driver_mut(driver_id)
            .ok_or(DispatchError::DriverNotFound(*driver_id))?;

        driver.current_location = new_location;
        Ok(())
    }
}
