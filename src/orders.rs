use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Food,        // 外卖
    Express,     // 快递
    FreshFood,   // 生鲜
    RideHailing, // 打车/顺风车
    Cargo,       // 货运
}

impl OrderType {
    pub fn display_name(&self) -> &str {
        match self {
            OrderType::Food => "外卖订单",
            OrderType::Express => "快递订单",
            OrderType::FreshFood => "生鲜订单",
            OrderType::RideHailing => "打车订单",
            OrderType::Cargo => "货运订单",
        }
    }

    pub fn requires_large_vehicle(&self) -> bool {
        matches!(self, OrderType::Express | OrderType::Cargo)
    }

    pub fn is_time_sensitive(&self) -> bool {
        matches!(
            self,
            OrderType::Food | OrderType::FreshFood | OrderType::RideHailing
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Dispatched,
    PickedUp,
    InTransit,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub pickup_location: Location,
    pub delivery_location: Location,
    pub distance_km: f64,
    pub estimated_duration_minutes: u32,
    pub price: f64,
    pub created_at: i64,
    pub assigned_to: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

impl Order {
    pub fn new(
        order_type: OrderType,
        pickup_location: Location,
        delivery_location: Location,
    ) -> Self {
        let distance_km = distance_km(pickup_location, delivery_location);
        let estimated_duration_minutes = Self::estimate_duration(distance_km, &order_type);
        let price = Self::calculate_price(distance_km, &order_type);

        Self {
            id: Uuid::new_v4(),
            order_type,
            status: OrderStatus::Pending,
            pickup_location,
            delivery_location,
            distance_km,
            estimated_duration_minutes,
            price,
            created_at: chrono::Utc::now().timestamp(),
            assigned_to: None,
            metadata: HashMap::new(),
        }
    }

    fn estimate_duration(distance_km: f64, order_type: &OrderType) -> u32 {
        let base_speed = match order_type {
            OrderType::Food | OrderType::FreshFood => 20.0, // 20 km/h
            OrderType::Express => 30.0,                     // 30 km/h
            OrderType::RideHailing => 40.0,                 // 40 km/h
            OrderType::Cargo => 35.0,                       // 35 km/h
        };

        let hours = distance_km / base_speed;
        let minutes = (hours * 60.0) as u32;
        minutes.max(5) // 至少 5 分钟
    }

    fn calculate_price(distance_km: f64, order_type: &OrderType) -> f64 {
        let base_price = match order_type {
            OrderType::Food => 5.0,
            OrderType::Express => 8.0,
            OrderType::FreshFood => 6.0,
            OrderType::RideHailing => 10.0,
            OrderType::Cargo => 15.0,
        };

        let per_km_price = match order_type {
            OrderType::Food => 2.0,
            OrderType::Express => 3.0,
            OrderType::FreshFood => 2.5,
            OrderType::RideHailing => 2.8,
            OrderType::Cargo => 5.0,
        };

        (base_price + distance_km * per_km_price).round()
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
