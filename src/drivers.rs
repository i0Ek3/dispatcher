use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriverStatus {
    Idle,    // 空闲
    Busy,    // 忙碌
    Offline, // 离线
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VehicleType {
    ElectricBike, // 电动车
    Motorcycle,   // 摩托车
    Car,          // 汽车
    Van,          // 面包车
    Truck,        // 卡车
}

impl VehicleType {
    pub fn display_name(&self) -> &str {
        match self {
            VehicleType::ElectricBike => "电动车",
            VehicleType::Motorcycle => "摩托车",
            VehicleType::Car => "汽车",
            VehicleType::Van => "面包车",
            VehicleType::Truck => "卡车",
        }
    }

    pub fn capacity(&self) -> usize {
        match self {
            VehicleType::ElectricBike => 2,
            VehicleType::Motorcycle => 3,
            VehicleType::Car => 4,
            VehicleType::Van => 6,
            VehicleType::Truck => 10,
        }
    }

    pub fn can_carry_large_items(&self) -> bool {
        matches!(
            self,
            VehicleType::Car | VehicleType::Van | VehicleType::Truck
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub id: Uuid,
    pub name: String,
    pub status: DriverStatus,
    pub vehicle_type: VehicleType,
    pub current_location: Location,
    pub capacity: usize,
    pub current_load: usize,
    pub rating: f64,
    pub total_orders: u32,
    pub metadata: HashMap<String, String>,
}

impl Driver {
    pub fn new(name: String, vehicle_type: VehicleType, current_location: Location) -> Self {
        let capacity = vehicle_type.capacity();

        Self {
            id: Uuid::new_v4(),
            name,
            status: DriverStatus::Idle,
            vehicle_type,
            current_location,
            capacity,
            current_load: 0,
            rating: 5.0,
            total_orders: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn is_available(&self) -> bool {
        self.status != DriverStatus::Offline && self.current_load < self.capacity
    }

    pub fn load_ratio(&self) -> f64 {
        if self.capacity == 0 {
            1.0
        } else {
            self.current_load as f64 / self.capacity as f64
        }
    }

    pub fn assign_order(&mut self) -> bool {
        if self.is_available() {
            self.current_load += 1;
            self.total_orders += 1;
            if self.current_load >= self.capacity {
                self.status = DriverStatus::Busy;
            }
            true
        } else {
            false
        }
    }

    pub fn release_order(&mut self) {
        if self.current_load > 0 {
            self.current_load -= 1;
            if self.current_load < self.capacity && self.status != DriverStatus::Offline {
                self.status = DriverStatus::Idle;
            }
        }
    }

    pub fn distance_to(&self, location: Location) -> f64 {
        distance_km(self.current_location, location)
    }
}

#[derive(Debug)]
pub struct DriverPool {
    drivers: HashMap<Uuid, Driver>,
}

impl DriverPool {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }

    pub fn add_driver(&mut self, driver: Driver) {
        self.drivers.insert(driver.id, driver);
    }

    pub fn remove_driver(&mut self, driver_id: &Uuid) -> Option<Driver> {
        self.drivers.remove(driver_id)
    }

    pub fn get_driver(&self, driver_id: &Uuid) -> Option<&Driver> {
        self.drivers.get(driver_id)
    }

    pub fn get_driver_mut(&mut self, driver_id: &Uuid) -> Option<&mut Driver> {
        self.drivers.get_mut(driver_id)
    }

    pub fn get_all_drivers(&self) -> Vec<&Driver> {
        self.drivers.values().collect()
    }

    pub fn get_available_drivers(&self) -> Vec<&Driver> {
        self.drivers.values().filter(|d| d.is_available()).collect()
    }
}

impl Default for DriverPool {
    fn default() -> Self {
        Self::new()
    }
}
