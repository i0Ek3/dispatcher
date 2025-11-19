use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

/// 计算两个位置之间的距离（公里）
/// 使用 Haversine 公式
pub fn distance_km(loc1: Location, loc2: Location) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = loc1.latitude.to_radians();
    let lat2_rad = loc2.latitude.to_radians();
    let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
    let delta_lng = (loc2.longitude - loc1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}
