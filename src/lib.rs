use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod drivers;
pub mod engine;
pub mod location;
pub mod orders;
pub mod rules;
pub mod strategies;

pub use drivers::{Driver, DriverPool, DriverStatus, VehicleType};
pub use engine::{DispatchEngine, DispatchError, DispatchResult};
pub use location::{Location, distance_km};
pub use orders::{Order, OrderStatus, OrderType};
pub use rules::{CapacityRule, DispatchRule, DistanceRule, RatingRule, VehicleTypeRule};
pub use strategies::{
    CapacityMatchStrategy, DispatchStrategy, LoadBalancedStrategy, NearestFirstStrategy,
    RatingPriorityStrategy,
};
