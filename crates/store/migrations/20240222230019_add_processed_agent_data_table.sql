-- Add migration script here
CREATE TYPE ROAD_STATE AS ENUM ('Smooth', 'Rough');

CREATE TABLE processed_agent_data(
    id BIGSERIAL PRIMARY KEY NOT NULL,
    road_state ROAD_STATE NOT NULL,
    x FLOAT NOT NULL,
    y FLOAT NOT NULL,
    z FLOAT NOT NULL,
    latitude FLOAT NOT NULL,
    longitude FLOAT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);