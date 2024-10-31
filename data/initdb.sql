CREATE DATABASE rustwebdev;

GRANT ALL PRIVILEGES ON DATABASE rustwebdev TO postgres;
GRANT ALL ON SCHEMA public TO postgres;

ALTER USER postgres WITH PASSWORD 'postgres';
