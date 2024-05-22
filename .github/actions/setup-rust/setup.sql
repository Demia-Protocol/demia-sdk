use messages;

CREATE TABLE IF NOT EXISTS app (
    app_id VARBINARY(255) NOT NULL PRIMARY KEY
);

-- Create the 'messages' table
CREATE TABLE IF NOT EXISTS sql_messages (
    msg_id VARBINARY(255) NOT NULL,
    raw_content LONGBLOB NOT NULL,
    timestamp DATETIME NOT NULL,
    public_key VARBINARY(255) NOT NULL,
    signature VARBINARY(255) NOT NULL,
    app_id VARBINARY(255) NOT NULL,
    PRIMARY KEY (msg_id, app_id),
    FOREIGN KEY (app_id) REFERENCES app(app_id)
);