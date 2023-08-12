-- Add migration script here
-- Create contacts table
CREATE TABLE contacts (                                                                                 
    id SERIAL PRIMARY KEY,
    first VARCHAR(255) NOT NULL,
    last VARCHAR(255),
    phone VARCHAR(32) NOT NULL,
    email VARCHAR(255) NOT NULL
);
