CREATE TABLE player_data (
    uuid UUID NOT NULL,
    namespace VARCHAR(64) NOT NULL,
    path VARCHAR(128) NOT NULL,
    value JSONB NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (uuid, namespace, path)
);

CREATE INDEX idx_player_data_uuid ON player_data (uuid);
CREATE INDEX idx_player_data_lookup ON player_data (uuid, namespace, path);
