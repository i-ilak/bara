use reqwest;
use serde::{Deserialize, Deserializer, Serialize};
use std::error::Error as StdError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceView {
    #[serde(deserialize_with = "deserialize_location")]
    pub location: Coordinate,
    pub zoom: u8,
}

fn deserialize_location<'de, D>(deserializer: D) -> Result<Coordinate, D::Error>
where
    D: Deserializer<'de>,
{
    let coords: Vec<f64> = Vec::deserialize(deserializer)?;
    if coords.len() != 2 {
        return Err(serde::de::Error::custom(
            "Location must be a sequence of two numbers",
        ));
    }
    Ok(Coordinate {
        longitude: coords[0],
        latitude: coords[1],
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDataSize {
    width: String,
    height: String,
}

#[derive(Debug, Serialize)]
pub struct MapData {
    pub source_view: SourceView,
    pub size: MapDataSize,
    pub waypoints: Vec<Coordinate>,
    pub route: Option<Vec<Coordinate>>,
}

impl<'de> Deserialize<'de> for MapData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMapData {
            source_view: SourceView,
            size: MapDataSize,
            waypoints: Vec<Vec<f64>>,
        }

        let raw = RawMapData::deserialize(deserializer)?;

        let waypoints = raw
            .waypoints
            .iter()
            .map(|point| {
                if point.len() != 2 {
                    return Err(serde::de::Error::custom(
                        "Each waypoint must have exactly 2 coordinates",
                    ));
                }
                Ok(Coordinate {
                    longitude: point[1],
                    latitude: point[0],
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let route = fetch_route(&waypoints).unwrap();

        Ok(MapData {
            source_view: raw.source_view,
            size: raw.size,
            waypoints: waypoints,
            route: Some(route),
        })
    }
}

/// Fetches a route from the OSMR demo server based on the given waypoints.
///
/// # Arguments
/// * `waypoints` - A vector of waypoints where each waypoint is a vector of [longitude, latitude].
///
/// # Returns
/// A vector of route geometry coordinates as vectors of [longitude, latitude].
pub fn fetch_route(waypoints: &Vec<Coordinate>) -> Result<Vec<Coordinate>, Box<dyn StdError>> {
    const OSMR_URL: &str = "https://router.project-osrm.org/route/v1/driving/";

    if waypoints.len() < 2 {
        return Err("At least two waypoints are required for routing.".into());
    }

    let coordinates = waypoints
        .iter()
        .map(|point| format!("{},{}", point.longitude, point.latitude))
        .collect::<Vec<_>>()
        .join(";");
    let url = format!(
        "{}{}?overview=full&geometries=geojson",
        OSMR_URL, coordinates
    );

    let response = reqwest::blocking::get(&url)?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json()?;

        let route_geometry = data["routes"][0]["geometry"]["coordinates"]
            .as_array()
            .ok_or("Invalid GeoJSON format: coordinates not found")?;

        let coordinates = route_geometry
            .iter()
            .map(|coord| {
                let lon = coord[1].as_f64().ok_or("Invalid longitude")?;
                let lat = coord[0].as_f64().ok_or("Invalid latitude")?;
                Ok(Coordinate {
                    longitude: lon,
                    latitude: lat,
                })
            })
            .collect::<Result<Vec<_>, &str>>()?;

        Ok(coordinates)
    } else {
        Err(format!(
            "Failed to fetch route. HTTP {}: {}",
            response.status(),
            response.text()?
        )
        .into())
    }
}
