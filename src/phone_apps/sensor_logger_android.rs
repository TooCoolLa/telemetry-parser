use std::io::*;

use crate::tags_impl::*;
use crate::*;
use memchr::memmem;

pub fn detect(buffer: &[u8], _filename: &str) -> bool {
    memmem::find(buffer, b"time,seconds_elapsed,z,y,x").is_some()
}

pub fn parse<T: Read + Seek>(stream: &mut T, _size: usize) -> Result<Vec<SampleInfo>> {
    let mut gyro = Vec::new();
    let mut accl = Vec::new();
    let mut magn = Vec::new();
    
    let mut last_timestamp = 0.0;
    let mut first_timestamp = 0.0;

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(stream);
    
    let h = csv.headers()?.clone();
    for row in csv.records() {
        let row = row?;
        let map = util::create_csv_map_hdr(&row, &h);

        let mut ts = map.get("time").unwrap_or(&"0.0").parse::<f64>().unwrap_or(0.0); // seconds since UNIX epoch
        if first_timestamp == 0.0 {
            first_timestamp = ts;
        }
        last_timestamp = ts;
        ts -= first_timestamp;
        ts *= 1.0e-9; // nanoseconds to seconds

        // TODO implement this
        crate::try_block!({
            accl.push(TimeVector3 {
                t: ts as f64,
                x: map.get("AccelerationX")?.parse::<f64>().ok()?,
                y: map.get("AccelerationY")?.parse::<f64>().ok()?,
                z: map.get("AccelerationZ")?.parse::<f64>().ok()?
            });
        });
        crate::try_block!({
            gyro.push(TimeVector3 {
                t: ts as f64,
                x: map.get("x")?.parse::<f64>().ok()?,
                y: map.get("y")?.parse::<f64>().ok()?,
                z: map.get("z")?.parse::<f64>().ok()?
            });
        });

        // TODO implement this
        crate::try_block!({
            magn.push(TimeVector3 {
                t: ts as f64,
                x: map.get("MagneticFieldX")?.parse::<f64>().ok()?,
                y: map.get("MagneticFieldY")?.parse::<f64>().ok()?,
                z: map.get("MagneticFieldZ")?.parse::<f64>().ok()? 
            });
        });
    }

    let mut map = GroupedTagMap::new();

    util::insert_tag(&mut map, tag!(parsed GroupId::Accelerometer, TagId::Data, "Accelerometer data", Vec_TimeVector3_f64, |v| format!("{:?}", v), accl, vec![]));
    util::insert_tag(&mut map, tag!(parsed GroupId::Accelerometer, TagId::Unit, "Accelerometer unit", String, |v| v.to_string(), "m/s²".into(),  Vec::new()));

    util::insert_tag(&mut map, tag!(parsed GroupId::Gyroscope,     TagId::Data, "Gyroscope data",     Vec_TimeVector3_f64, |v| format!("{:?}", v), gyro, vec![]));
    util::insert_tag(&mut map, tag!(parsed GroupId::Gyroscope,     TagId::Unit, "Gyroscope unit",     String, |v| v.to_string(), "rad/s".into(), Vec::new()));

    let imu_orientation = "XYZ"; // TODO
    util::insert_tag(&mut map, tag!(parsed GroupId::Accelerometer, TagId::Orientation, "IMU orientation", String, |v| v.to_string(), imu_orientation.into(), Vec::new()));
    util::insert_tag(&mut map, tag!(parsed GroupId::Gyroscope,     TagId::Orientation, "IMU orientation", String, |v| v.to_string(), imu_orientation.into(), Vec::new()));

    util::insert_tag(&mut map, tag!(parsed GroupId::Magnetometer,  TagId::Data, "Magnetometer data", Vec_TimeVector3_f64, |v| format!("{:?}", v), magn, vec![]));
    util::insert_tag(&mut map, tag!(parsed GroupId::Magnetometer,  TagId::Unit, "Magnetometer unit", String, |v| v.to_string(), "μT".into(), Vec::new()));

    Ok(vec![
        SampleInfo { index: 0, timestamp_ms: first_timestamp as f64, duration_ms: (last_timestamp - first_timestamp) as f64, tag_map: Some(map) }
    ])
}
