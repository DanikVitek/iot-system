use iot_system::domain::{Agent, ProcessedAgent, RoadState};

pub fn process_agent_data(current_data: Agent, prev_data: Option<&Agent>) -> ProcessedAgent {
    let road_state = 'b: {
        let Some(prev_data) = prev_data else {
            break 'b RoadState::default();
        };
        let dt = current_data
            .timestamp()
            .signed_duration_since(prev_data.timestamp())
            .num_seconds() as f64; // seconds
        let a1_z = prev_data.accelerometer().z(); // mm/s^2
        let a2_z = current_data.accelerometer().z(); // mm/s^2
        let dv_z = (a1_z + a2_z) / 2.0 * dt; // mm/s
        if dv_z.abs() > 100.0 {
            RoadState::Rough
        } else {
            RoadState::Smooth
        }
    };

    ProcessedAgent::new(current_data, road_state)
}
