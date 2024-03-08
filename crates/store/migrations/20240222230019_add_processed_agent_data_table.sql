-- Add migration script here
CREATE TABLE processed_agent_data(
    id SERIAL PRIMARY KEY NOT NULL,
    road_state VARCHAR(255) NOT NULL,
    x FLOAT NOT NULL,
    y FLOAT NOT NULL,
    z FLOAT NOT NULL,
    latitude FLOAT NOT NULL,
    longitude FLOAT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
)