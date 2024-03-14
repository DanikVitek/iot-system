use iot_system::domain::{Agent, ProcessedAgent, RoadState};

#[tracing::instrument]
pub fn process_agent_data(current_data: Agent, prev_data: Option<&Agent>) -> ProcessedAgent {
    let road_state = 'b: {
        let Some(prev_data) = prev_data else {
            break 'b RoadState::default();
        };
        let dt = current_data
            .timestamp()
            .signed_duration_since(prev_data.timestamp())
            .num_milliseconds() as f64
            / 1000.0; // seconds
        let a1_z = prev_data.accelerometer().z(); // mm/s^2
        let a2_z = current_data.accelerometer().z(); // mm/s^2
        let da_z = a2_z - a1_z; // mm/s^2
        let da_z_dt = da_z / dt; // mm/s^3
        tracing::debug!("da_z_dt: {}", da_z_dt);
        if da_z_dt.abs() > 1000.0 {
            RoadState::Rough
        } else {
            RoadState::Smooth
        }
    };

    ProcessedAgent::new(current_data, road_state)
}
