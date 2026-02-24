use geohash::Coord;

/// Encode latitude/longitude to a geohash string at the given precision.
pub fn encode(lat: f64, lon: f64, precision: u8) -> Result<String, geohash::GeohashError> {
    let coord = Coord { x: lon, y: lat };
    geohash::encode(coord, precision as usize)
}

/// Decode a geohash string back to (lat, lon) center point.
pub fn decode(hash: &str) -> Result<(f64, f64), geohash::GeohashError> {
    let (coord, _, _) = geohash::decode(hash)?;
    Ok((coord.y, coord.x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let lat = 60.1699;
        let lon = 24.9384;
        let hash = encode(lat, lon, 8).unwrap();
        assert_eq!(hash.len(), 8);

        let (dlat, dlon) = decode(&hash).unwrap();
        assert!((dlat - lat).abs() < 0.001);
        assert!((dlon - lon).abs() < 0.001);
    }

    #[test]
    fn precision_affects_length() {
        let h4 = encode(60.17, 24.94, 4).unwrap();
        let h12 = encode(60.17, 24.94, 12).unwrap();
        assert_eq!(h4.len(), 4);
        assert_eq!(h12.len(), 12);
    }
}
